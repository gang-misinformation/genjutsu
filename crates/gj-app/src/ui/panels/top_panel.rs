use egui::{Color32, Context, RichText};
use crate::events::AppEvent;
use crate::ui::UiEventSender;

#[derive(Default)]
pub struct TopPanel {
    // local state / child components may go here
}

impl TopPanel {
    pub fn show(&mut self, ctx: &Context, sender: &mut UiEventSender) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 3D Generation Studio");
                ui.separator();
                ui.label(RichText::new("Status:").color(Color32::LIGHT_BLUE));
                // status display would be written by side panel pushing AppEvent::Status
            });
        });
    }

    pub fn on_app_event(&mut self, _ev: &AppEvent) {
        // react to app events if needed (e.g. update internal text)
    }
}