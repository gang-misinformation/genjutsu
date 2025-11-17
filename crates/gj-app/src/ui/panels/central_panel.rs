use egui::{Color32, Context, RichText};
use crate::events::AppEvent;
use crate::ui::UiEventSender;

#[derive(Default)]
pub struct CentralPanel {}

impl CentralPanel {
    pub fn show(&mut self, ctx: &Context, _sender: &mut UiEventSender) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Always allocate space to prevent zero-size viewport issues
                ui.allocate_space(ui.available_size());

                // Show instructions centered
                ui.vertical_centered(|ui| {
                    ui.label("Viewport - 3D scene renders under the UI.");
                    ui.label("When no cloud is loaded, this area shows instructions.");
                });
            });
    }

    pub fn on_app_event(&mut self, _ev: &AppEvent) {}
}