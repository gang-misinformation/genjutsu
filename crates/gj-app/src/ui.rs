mod panels;

use std::sync::Arc;
use egui::{Context, FullOutput};
use winit::window::Window;
use crate::events::{AppEvent, UiEvent};
use crate::gfx::GfxState;
use crate::ui::panels::Panels;

pub struct UiState {
    pub(crate) egui_state: egui_winit::State,
    pub(crate) egui_ctx: egui::Context,
    pub(crate) egui_renderer: egui_wgpu::Renderer,

    ui_outgoing: Vec<UiEvent>,
    app_incoming: Vec<AppEvent>,

    app_event_tx: std::sync::mpsc::Sender<AppEvent>,

    panels: Panels,
}

impl UiState {
    pub fn new(gfx: &GfxState, window: Arc<Window>) -> Self {
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &gfx.device,
            gfx.config.format,
            egui_wgpu::RendererOptions::default()
        );

        let (tx, rx) = std::sync::mpsc::channel::<AppEvent>();

        Self {
            egui_ctx,
            egui_state,
            egui_renderer,
            ui_outgoing: Vec::new(),
            app_incoming: Vec::new(),
            app_event_tx: tx,
            panels: Panels::default(),
        }
    }

    pub fn on_window_event(&mut self, window: &winit::window::Window, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub fn draw(&mut self, window: &winit::window::Window) -> (egui::FullOutput, Vec<UiEvent>) {
        let raw_input = self.egui_state.take_egui_input(window);
        let mut sender = UiEventSender::default();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.panels.draw(ctx, &mut sender);
        });

        let events = sender.take_events();
        (full_output, events)
    }

    fn draw_panels(&mut self, ctx: &Context) {
        let mut sender = UiEventSender::default();
        self.panels.draw(ctx, &mut sender);

        // merge events
        self.ui_outgoing.extend(sender.take_events());
    }

    pub fn push_app_event(&mut self, ev: AppEvent) {
        self.app_incoming.push(ev);
    }

    pub fn take_ui_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.ui_outgoing)
    }

    pub fn app_event_sender_clone(&self) -> std::sync::mpsc::Sender<AppEvent> {
        self.app_event_tx.clone()
    }

    /// internal: call after draw_ui to merge events and broadcast app_incoming to panels
    pub fn after_draw_process(&mut self, full_output: egui::FullOutput, events_from_draw: Vec<UiEvent>) {
        // collect outgoing ui events
        self.ui_outgoing.extend(events_from_draw);

        // broadcast app events to all panels (so child components can react)
        for app_ev in self.app_incoming.drain(..) {
            self.panels.on_app_event(&app_ev);
        }

        // handle platform output (clipboard, window title, etc.) is done by caller
    }
}

#[derive(Default)]
pub struct UiEventSender {
    events: Vec<UiEvent>,
}

impl UiEventSender {
    pub fn instant(&mut self, e: UiEvent) {
        self.events.push(e);
    }
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.events)
    }
}

pub trait UiComponent {
    fn show(&mut self, ctx: &Context, sender: &mut UiEventSender);

    fn on_app_event(&mut self, _ev: &AppEvent) {}
}
