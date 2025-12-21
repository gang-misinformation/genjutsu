use egui::{Color32, Context, RichText};
use crate::events::AppEvent;
use crate::ui::{UiComponent, UiContext, UiEventSender};

#[derive(Default)]
pub struct TopPanel {

}

impl UiComponent for TopPanel {
    fn show(&mut self, ctx: &Context, sender: &mut UiEventSender, ui_ctx: &UiContext) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 genjutsu");
                ui.separator();
                ui.label(RichText::new("Status:").color(Color32::LIGHT_BLUE));
            });
        });
    }

    fn on_app_event(&mut self, _ev: &AppEvent) {}
}