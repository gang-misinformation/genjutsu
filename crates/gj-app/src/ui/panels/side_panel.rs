use egui::{Context, RichText};
use crate::events::{AppEvent, UiEvent};
use crate::ui::UiEventSender;

#[derive(Default)]
pub struct SidePanel {
    // example child state (render settings)
    pub show_grid: bool,
    pub last_status: Option<String>,
}

impl SidePanel {
    pub fn show(&mut self, ctx: &Context, sender: &mut UiEventSender) {
        egui::SidePanel::left("side_panel").default_width(320.0).show(ctx, |ui| {
            ui.heading("Pipeline");
            ui.separator();

            ui.label("Select 4 multi-view images:");
            if ui.button("📁 Load Images...").clicked() {
                sender.instant(UiEvent::LoadImages);
            }

            ui.separator();
            ui.checkbox(&mut self.show_grid, "Show Grid");
            // if you want to forward show_grid changes to app:
            if ui.button("Apply Grid Toggle").clicked() {
                sender.instant(UiEvent::ToggleWireframe(self.show_grid));
            }

            ui.separator();
            if let Some(ref s) = self.last_status {
                ui.label(RichText::new(format!("Status: {}", s)).color(egui::Color32::LIGHT_BLUE));
            }

            ui.separator();
            ui.heading("Camera Controls");
            ui.label("• Left drag: Rotate");
            ui.label("• Mouse wheel: Zoom");
            if ui.button("Reset Camera").clicked() {
                sender.instant(UiEvent::ResetCamera);
            }
        });
    }

    pub fn on_app_event(&mut self, ev: &AppEvent) {
        match ev {
            AppEvent::Status(s) => self.last_status = Some(s.clone()),
            AppEvent::Progress(p) => self.last_status = Some(format!("Loading {:.0}%", p * 100.0)),
            AppEvent::SceneReady => self.last_status = Some("Scene ready".into()),
            _ => {}
        }
    }
}