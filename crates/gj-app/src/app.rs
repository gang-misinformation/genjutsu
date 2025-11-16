use std::sync::Arc;
use egui_wgpu::wgpu;
use winit::{
    event::*,
    event_loop::ActiveEventLoop,
};
use winit::application::ApplicationHandler;
use winit::window::{WindowAttributes, WindowId};
use crate::events::GjEvent;
use crate::state::AppState;
use crate::ui;
use crate::ui::UiState;

#[derive(Default)]
pub struct App {
    state: Option<AppState>
}

impl ApplicationHandler<GjEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Gaussian Splatting Viewer")
            .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(AppState::new(window.clone())).unwrap();
        self.state = Some(state);
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: GjEvent) {
        if let Some(state) = &mut self.state {
            match event {
                GjEvent::Ui(e) => {

                }
                GjEvent::App(e) => {
                    state.ui.push_app_event(e);
                }
            }
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        if state.window.id() != window_id {
            return;
        }

        // Let egui handle the event first
        let response = state.ui.egui_state.on_window_event(&state.window, &event);

        if response.repaint {
            state.window.request_redraw();
        }

        // Handle events not consumed by egui
        if !response.consumed {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    state.update();
                    state.render();
                }
                _ => {
                    state.input(&event);
                }
            }
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}
