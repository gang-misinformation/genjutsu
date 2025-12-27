use std::collections::HashMap;
use std::sync::Arc;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::StoreOp;
use winit::event::WindowEvent;
use winit::window::Window;
use chrono::Utc;
use log::info;
use surrealdb::types::Datetime as SurrealDatetime;
use surrealdb_types::{RecordIdKey, ToSql};
use winit::event_loop::EventLoopProxy;
use gj_core::gaussian_cloud::GaussianCloud;
use gj_splat::camera::Camera;
use gj_splat::renderer::GaussianRenderer;
use crate::generator::db::job::JobRecord;
use crate::events::{AppEvent, GenEvent, GjEvent};
use crate::generator::Generator;
use crate::gfx::GfxState;
use crate::job::{JobMetadata, JobOutputs, JobStatus};
use crate::ui;
use crate::ui::{UiEvent, UiState};

pub struct AppState {
    pub(crate) window: Arc<Window>,
    event_loop_proxy: Arc<EventLoopProxy<GjEvent>>,

    pub gfx: GfxState,
    pub ui: UiState,

    // 3D renderer state
    pub renderer: GaussianRenderer,
    pub camera: Camera,
    pub gaussian_cloud: Option<GaussianCloud>,

    // App-side state exposed to UI
    pub prompt: String,
    pub status: String,

    // Mouse state
    pub mouse_pressed: bool,
    pub last_mouse_pos: Option<(f32, f32)>,

    pub(crate) generator: Generator,

    // In-memory cache of active job progress (not persisted)
    pub active_job_progress: HashMap<String, (JobMetadata, Option<JobOutputs>)>,
}

impl AppState {
    pub async fn new(window: Arc<Window>, event_loop_proxy: Arc<EventLoopProxy<GjEvent>>) -> anyhow::Result<Self> {
        let generator = Generator::new(event_loop_proxy.clone()).await?;

        let gfx = GfxState::new(window.clone()).await?;
        let mut ui_state = UiState::new(&gfx, window.clone(), event_loop_proxy.clone());

        ui_state.add_component(Box::new(ui::CentralPanel::default()));
        ui_state.add_component(Box::new(ui::SidePanel::default()));
        ui_state.add_component(Box::new(ui::TopPanel::default()));
        ui_state.add_component(Box::new(ui::QueuePanel::default()));

        let renderer = GaussianRenderer::new(
            gfx.device.clone(),
            gfx.queue.clone(),
            gfx.config.format
        ).await;

        let mut camera = Camera::default();
        let size = window.inner_size();
        camera.aspect_ratio = size.width as f32 / size.height as f32;

        let mut state = Self {
            window,
            event_loop_proxy,
            renderer,
            camera,
            gfx,
            ui: ui_state,
            gaussian_cloud: None,
            prompt: String::new(),
            status: "Ready".into(),
            mouse_pressed: false,
            last_mouse_pos: None,
            generator,
            active_job_progress: HashMap::new(),
        };

        // Load existing jobs from database and clean up stale states
        state.load_and_cleanup_jobs().await?;

        Ok(state)
    }

    /// Load all jobs from database and clean up stale GENERATING states
    async fn load_and_cleanup_jobs(&mut self) -> anyhow::Result<()> {
        let mut jobs = self.generator.get_jobs().await?;

        println!("Loaded {} jobs from database", jobs.len());

        // Clean up any jobs stuck in GENERATING or QUEUED state
        // (they were interrupted when the app closed)
        for job in &mut jobs {
            match job.metadata.status {
                JobStatus::GENERATING | JobStatus::QUEUED => {
                    println!("Cleaning up stale job: {:?} (was {:?})", job.id, job.metadata.status);

                    // Mark as failed due to interruption
                    let mut updated_metadata = job.metadata.clone();
                    updated_metadata.status = JobStatus::FAILED;
                    updated_metadata.error = Some("Job interrupted by application shutdown".to_string());
                    updated_metadata.completed_at = Some(SurrealDatetime::from(chrono::Utc::now()));
                    updated_metadata.updated_at = SurrealDatetime::from(chrono::Utc::now());

                    // Update in database using RecordId directly
                    self.generator.update_job_status_by_id(
                        job.id.clone(),
                        updated_metadata.clone(),
                        None
                    ).await?;

                    // Update local copy
                    job.metadata = updated_metadata;
                }
                _ => {}
            }
        }

        self.ui.set_jobs(jobs);
        Ok(())
    }

    /// Load all jobs from database and update UI
    async fn load_jobs(&mut self) -> anyhow::Result<()> {
        let jobs = self.generator.get_jobs().await?;
        println!("Loaded {} jobs from database", jobs.len());
        self.ui.set_jobs(jobs);
        Ok(())
    }

    pub fn push_event(&self, event: AppEvent) {
        self.event_loop_proxy.send_event(GjEvent::App(event)).unwrap();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gfx.resize(new_size);
            self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn reset_camera(&mut self) {
        self.camera = Camera::default();
        let size = self.window.inner_size();
        self.camera.aspect_ratio = size.width as f32 / size.height as f32;
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        use winit::event::{ElementState, MouseScrollDelta};

        match event {
            WindowEvent::MouseInput { state, .. } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                if !self.mouse_pressed {
                    self.last_mouse_pos = None;
                }
                true
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x as f32, position.y as f32);

                if self.mouse_pressed {
                    if let Some((lx, ly)) = self.last_mouse_pos {
                        let dx = pos.0 - lx;
                        let dy = pos.1 - ly;
                        self.camera.rotate(dx * 0.1, -dy * 0.1);
                    }
                }

                self.last_mouse_pos = Some(pos);
                true
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 10.0,
                };

                self.camera.zoom(-scroll * 0.1);
                true
            }

            _ => false,
        }
    }

    pub fn load_gaussian_cloud(&mut self, cloud: GaussianCloud) {
        // Compute bounds
        let bounds = cloud.bounds();
        let center = bounds.center();
        let size = bounds.size();
        let max_dim = size[0].max(size[1]).max(size[2]);

        println!("Mesh bounds:");
        println!("  Center: [{:.3}, {:.3}, {:.3}]", center[0], center[1], center[2]);
        println!("  Size: [{:.3}, {:.3}, {:.3}]", size[0], size[1], size[2]);
        println!("  Max dimension: {:.3}", max_dim);

        // Auto-adjust camera distance based on mesh size
        self.camera.distance = max_dim * 2.5;
        self.camera.target = glam::Vec3::new(center[0], center[1], center[2]);
        self.camera.update_position();

        self.renderer.load_gaussians(&cloud);
        self.gaussian_cloud = Some(cloud);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        let output = self.gfx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.gfx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        // 3D scene
        if let Some(ref _cloud) = self.gaussian_cloud {
            let size = self.window.inner_size();
            self.renderer.render(
                &mut encoder,
                &view,
                &self.gfx.depth_view,
                &self.camera,
                (size.width, size.height),
            );
        } else {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }

        // UI
        let full_output = self.ui.draw(&self.window);

        let platform_output = full_output.platform_output.clone();
        self.ui.egui_state.handle_platform_output(&self.window, platform_output);

        let shapes = full_output.shapes.clone();
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = self.ui.egui_ctx.tessellate(shapes, pixels_per_point);

        let size = self.window.inner_size();
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, delta) in &full_output.textures_delta.set {
            self.ui.egui_renderer.update_texture(&self.gfx.device, &self.gfx.queue, *id, delta);
        }

        self.ui.egui_renderer.update_buffers(
            &self.gfx.device,
            &self.gfx.queue,
            &mut encoder,
            &paint_jobs,
            &screen_desc,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            self.ui.egui_renderer.render(&mut rpass.forget_lifetime(), &paint_jobs, &screen_desc);
        }

        for id in &full_output.textures_delta.free {
            self.ui.egui_renderer.free_texture(id);
        }

        self.gfx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn on_ui_event(&mut self, event: UiEvent) {
        pollster::block_on(
            async {
                match event {
                    UiEvent::GenerateWithModel { prompt, model } => {
                        self.generator.submit_job(prompt, model).await?;
                        self.load_jobs().await?;
                    }
                    UiEvent::ResetCamera => {
                        self.reset_camera();
                        self.push_event(AppEvent::Status("Camera reset".into()));
                    }
                    UiEvent::RemoveJob(id) => {
                        self.generator.remove_job(id).await?;
                        self.load_jobs().await?;
                    }
                    UiEvent::LoadScene(id) => {
                        self.load_scene_by_id(id).await?;
                    }
                    UiEvent::ClearCompletedJobs => {
                        self.generator.clear_completed().await?;
                        self.load_jobs().await?;
                    }
                    _ => {}
                }
                anyhow::Ok(())
            }
        ).unwrap();
    }

    /// Handle job status updates from Python worker
    pub async fn on_gen_event(&mut self, event: GenEvent) -> anyhow::Result<()> {
        match event {
            GenEvent::JobStatus { id, data, outputs } => {
                info!("Job status update: {} - {:?}", id, data.status);

                // IMPORTANT: Only write to database for terminal states
                match data.status {
                    JobStatus::COMPLETE | JobStatus::FAILED => {
                        // Terminal state - persist to database
                        self.generator.update_job_status(id.clone(), data.clone(), outputs.clone()).await?;

                        // Remove from in-memory cache
                        self.active_job_progress.remove(&id);

                        // Refresh UI from database
                        self.load_jobs().await?;

                        // Auto-load if complete
                        if data.status == JobStatus::COMPLETE {
                            if let Some(ref job_outputs) = outputs {
                                info!("Job complete! PLY at: {}", job_outputs.ply_path);
                                self.load_scene_from_path(&job_outputs.ply_path).await?;
                            }
                        }
                    }
                    JobStatus::GENERATING => {
                        // First GENERATING update - write to DB to mark job as started
                        if !self.active_job_progress.contains_key(&id) {
                            self.generator.update_job_status(id.clone(), data.clone(), outputs.clone()).await?;
                            self.load_jobs().await?;
                        }

                        // All subsequent updates - memory cache only
                        self.active_job_progress.insert(id.clone(), (data, outputs));

                        // Update UI directly without hitting database
                        self.update_ui_job_progress(id.clone(), self.active_job_progress.get(&id).unwrap().clone());
                    }
                    JobStatus::QUEUED => {
                        // Queued state is already written when job is submitted
                        // Just update UI cache
                        self.active_job_progress.insert(id.clone(), (data, outputs));
                    }
                }
            }
        }

        Ok(())
    }

    fn update_ui_job_progress(&mut self, job_id: String, data: (JobMetadata, Option<JobOutputs>)) {
        // Find the job in the UI state and update it in-place
        if let Some(job) = self.ui.ui_ctx.jobs.iter_mut().find(|j| {
            // Extract the ID part from RecordId for comparison
            match &j.id.key {
                RecordIdKey::String(id) => *id == job_id,
                _ => false
            }
        }) {
            job.metadata = data.0;
            if let Some(outputs) = data.1 {
                job.outputs = Some(outputs);
            }
        }
    }

    /// Load a scene by job ID
    async fn load_scene_by_id(&mut self, id: surrealdb_types::RecordId) -> anyhow::Result<()> {
        let jobs = self.generator.get_jobs().await?;

        if let Some(job) = jobs.iter().find(|j| j.id == id) {
            if let Some(ref outputs) = job.outputs {
                self.load_scene_from_path(&outputs.ply_path).await?;
                self.ui.ui_ctx.current_job_id = Some(id);
            } else {
                println!("Job has no outputs yet");
            }
        } else {
            println!("Job not found: {:?}", id);
        }

        Ok(())
    }

    /// Load a scene from a PLY file path
    async fn load_scene_from_path(&mut self, ply_path: &str) -> anyhow::Result<()> {
        println!("Loading scene from: {}", ply_path);

        // Convert relative path to absolute
        let path = std::env::current_dir()?.join(ply_path);

        if !path.exists() {
            anyhow::bail!("PLY file not found: {}", path.display());
        }

        // Load Gaussian cloud from PLY
        let cloud = GaussianCloud::from_ply(&path)?;
        println!("Loaded {} Gaussians from {}", cloud.count, path.display());

        // Load into renderer
        self.load_gaussian_cloud(cloud);

        self.push_event(AppEvent::SceneReady);

        Ok(())
    }
}