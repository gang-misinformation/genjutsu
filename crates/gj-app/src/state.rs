use std::sync::Arc;
use burn_ndarray::NdArray;
use burn_wgpu::Wgpu;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::StoreOp;
use winit::event::WindowEvent;
use winit::window::Window;

use gj_core::gaussian_cloud::GaussianCloud;
use gj_lgm::LGMPipeline;
use gj_splat::camera::Camera;
use gj_splat::renderer::GaussianRenderer;

use crate::events::{AppEvent, UiEvent};
use crate::gfx::GfxState;
use crate::ui::UiState;

pub struct AppState {
    pub(crate) window: Arc<Window>,

    pub gfx: GfxState,
    pub ui: UiState,

    // 3D renderer state
    pub renderer: GaussianRenderer,
    pub camera: Camera,
    pub gaussian_cloud: Option<GaussianCloud>,

    // LGM pipeline
    pub lgm_pipeline:  Arc<LGMPipeline<Wgpu>>,

    // App-side state exposed to UI
    pub prompt: String,
    pub status: String,

    // Mouse state
    pub mouse_pressed: bool,
    pub last_mouse_pos: Option<(f32, f32)>,

    // Tokio runtime for background tasks
    pub rt: tokio::runtime::Runtime,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gfx = GfxState::new(window.clone()).await?;
        let ui = UiState::new(&gfx, window.clone());

        let renderer = GaussianRenderer::new(
            gfx.device.clone(),
            gfx.queue.clone(),
            gfx.config.format
        ).await;

        let mut camera = Camera::default();
        let size = window.inner_size();
        camera.aspect_ratio = size.width as f32 / size.height as f32;

        let lgm_device = Default::default();
        let lgm_pipeline = Arc::new(LGMPipeline::new(lgm_device));

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;

        Ok(Self {
            window,
            renderer,
            camera,
            lgm_pipeline,
            gfx,
            ui,
            gaussian_cloud: None,

            prompt: String::new(),
            status: "Ready".into(),

            mouse_pressed: false,
            last_mouse_pos: None,

            rt,
        })
    }

    pub fn init(&mut self) {
        // Seed UI with initial state
        self.ui.push_app_event(AppEvent::Status(self.status.clone()));

        if self.gaussian_cloud.is_some() {
            self.ui.push_app_event(AppEvent::SceneReady);
        }
    }

    // --- Window resizing ----------------------------------------------------

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gfx.resize(new_size);
            self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    // --- Mouse + keyboard input --------------------------------------------

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

    // --- Event processing from UI ------------------------------------------

    pub fn update(&mut self) {
        let ui_events = self.ui.take_ui_events();

        for ev in ui_events {
            match ev {
                UiEvent::ResetCamera => {
                    self.camera = Camera::default();
                    let size = self.window.inner_size();
                    self.camera.aspect_ratio = size.width as f32 / size.height as f32;

                    self.ui.push_app_event(AppEvent::Status("Camera reset".into()));
                }

                UiEvent::ToggleWireframe(enabled) => {
                    self.ui.push_app_event(AppEvent::WireframeState(enabled));
                }

                UiEvent::LoadImages => {
                    let tx = self.ui.app_event_sender_clone();

                    let pipeline = self.lgm_pipeline.clone(); // clone if Arc; otherwise wrap in Arc earlier.

                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_files()
                    {
                        let images: Result<Vec<_>, _> = files.iter()
                            .map(|path| image::open(path).map(|img| img.to_rgba8()))
                            .collect();
                        let images = images.unwrap_or_else(|err| panic!("Failed to load images: {}", err));
                        self.status = "Generating 3D...".to_string();
                        let cloud = pipeline.generate(&images).unwrap();
                        self.renderer.load_gaussians(&cloud);
                        self.gaussian_cloud = Some(cloud.clone());

                        self.status = format!("Generated {} Gaussians", cloud.count);
                    }
                    self.rt.spawn(async move {
                        let _ = tx.send(AppEvent::SceneReady);
                    });
                }

                UiEvent::Log(msg) => {
                    self.ui.push_app_event(AppEvent::Log(format!("UI: {}", msg)));
                }
                _ => {}
            }
        }
    }

    // --- 3D rendering + UI rendering ---------------------------------------

    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.gfx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.gfx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        // --- 3D scene -------------------------------------------------------

        if let Some(ref cloud) = self.gaussian_cloud {
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

        // --- UI -------------------------------------------------------------

        let (full_output, ui_events) = self.ui.draw(&self.window);

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

        // draw UI pass
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

        // Merge UI events + broadcast to panels (child components)
        self.ui.after_draw_process(full_output, ui_events);

        Ok(())
    }
}
