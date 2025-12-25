use std::sync::Arc;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::StoreOp;
use winit::event::WindowEvent;
use winit::window::Window;
use chrono::Utc;
use surrealdb::types::Datetime as SurrealDatetime;
use winit::event_loop::EventLoopProxy;
use gj_core::gaussian_cloud::GaussianCloud;
use gj_splat::camera::Camera;
use gj_splat::renderer::GaussianRenderer;
use crate::generator::db::job::JobRecord;
use crate::events::{AppEvent, GjEvent};
use crate::generator::Generator;
use crate::gfx::GfxState;
use crate::job::JobStatus;
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

        Ok(Self {
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
            generator
        })
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
                        let jobs = self.generator.get_jobs().await?;
                        self.ui.set_jobs(jobs);
                    }
                    UiEvent::ResetCamera => {
                        self.reset_camera();
                        self.push_event(AppEvent::Status("Camera reset".into()));
                    }
                    UiEvent::RemoveJob(id) => {
                        self.generator.remove_job(id).await?;
                    },
                    _ => {}
                }
                anyhow::Ok(())
            }
        ).unwrap();
    }
}