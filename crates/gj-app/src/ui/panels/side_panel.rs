use egui::{Context, RichText, TextEdit};
use crate::events::{AppEvent, UiEvent};
use crate::ui::UiEventSender;

#[derive(Default)]
pub struct SidePanel {
    // Render settings
    pub show_grid: bool,
    pub last_status: Option<String>,

    // Prompt input
    pub prompt_text: String,
    pub is_generating: bool,
}

impl SidePanel {
    pub fn show(&mut self, ctx: &Context, sender: &mut UiEventSender) {
        egui::SidePanel::left("side_panel").default_width(320.0).show(ctx, |ui| {
            ui.heading("3D Generation");
            ui.separator();

            // === Text-to-3D Section ===
            ui.heading(RichText::new("✨ Generate from Text").size(16.0));
            ui.add_space(5.0);

            ui.label("Describe what you want to create:");

            let text_edit = TextEdit::multiline(&mut self.prompt_text)
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .hint_text("e.g., a red sports car, a blue crystal gem, a wooden chair...");

            ui.add(text_edit);

            ui.add_space(5.0);

            let generate_button = ui.add_enabled(
                !self.is_generating && !self.prompt_text.trim().is_empty(),
                egui::Button::new("🎨 Generate 3D Model")
            );

            if generate_button.clicked() {
                sender.instant(UiEvent::GenerateFromPrompt(self.prompt_text.clone()));
                self.is_generating = true;
            }

            ui.add_space(5.0);
            ui.label(RichText::new("Note: Currently using placeholder synthetic images").small().weak());

            ui.separator();

            // === Load from Files Section ===
            ui.heading(RichText::new("📁 Or Load from Files").size(16.0));
            ui.add_space(5.0);

            ui.label("Select 4 multi-view images:");

            let load_button = ui.add_enabled(
                !self.is_generating,
                egui::Button::new("📂 Load Images...")
            );

            if load_button.clicked() {
                sender.instant(UiEvent::LoadImages);
                self.is_generating = true;
            }

            ui.separator();

            // === Render Settings ===
            ui.heading("Render Settings");
            ui.checkbox(&mut self.show_grid, "Show Grid");

            if ui.button("Apply Grid Toggle").clicked() {
                sender.instant(UiEvent::ToggleWireframe(self.show_grid));
            }

            ui.separator();

            // === Status Display ===
            if let Some(ref s) = self.last_status {
                let status_color = if s.contains("Error") || s.contains("Failed") {
                    egui::Color32::from_rgb(255, 100, 100)
                } else if s.contains("Generated") || s.contains("ready") {
                    egui::Color32::from_rgb(100, 255, 100)
                } else {
                    egui::Color32::LIGHT_BLUE
                };

                ui.label(
                    RichText::new(format!("Status: {}", s))
                        .color(status_color)
                );
            }

            ui.separator();

            // === Camera Controls ===
            ui.heading("Camera Controls");
            ui.label("• Left drag: Rotate");
            ui.label("• Mouse wheel: Zoom");

            if ui.button("🔄 Reset Camera").clicked() {
                sender.instant(UiEvent::ResetCamera);
            }

            ui.add_space(10.0);

            // === Example Prompts ===
            ui.collapsing("💡 Example Prompts", |ui| {
                ui.label(RichText::new("Click to try:").weak());

                let examples = [
                    "a red sports car",
                    "a blue crystal gem",
                    "a yellow rubber duck",
                    "a green cactus plant",
                    "a purple alien creature",
                ];

                for example in examples {
                    if ui.button(example).clicked() {
                        self.prompt_text = example.to_string();
                    }
                }
            });
        });
    }

    pub fn on_app_event(&mut self, ev: &AppEvent) {
        match ev {
            AppEvent::Status(s) => {
                self.last_status = Some(s.clone());

                // Reset generating flag when done
                if s.contains("Generated") || s.contains("Error") || s.contains("Failed") || s.contains("ready") {
                    self.is_generating = false;
                }
            }
            AppEvent::Progress(p) => {
                self.last_status = Some(format!("Loading {:.0}%", p * 100.0));
            }
            AppEvent::SceneReady => {
                self.last_status = Some("Scene ready".into());
                self.is_generating = false;
            }
            AppEvent::GaussianCloudReady => {
                self.is_generating = false;
            }
            _ => {}
        }
    }
}