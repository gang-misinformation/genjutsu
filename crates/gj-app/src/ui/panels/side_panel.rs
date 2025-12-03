use egui::{Context, RichText, TextEdit, Color32};
use gj_core::Model3D;
use crate::events::{AppEvent, UiEvent};
use crate::ui::UiEventSender;

pub struct SidePanel {
    // Model selection (currently only Shap-E)
    pub selected_model: Model3D,

    // Status
    pub last_status: Option<String>,

    // Prompt input
    pub prompt_text: String,
    pub is_generating: bool,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            selected_model: Model3D::ShapE,
            last_status: None,
            prompt_text: String::new(),
            is_generating: false,
        }
    }
}

impl SidePanel {
    pub fn show(&mut self, ctx: &Context, sender: &mut UiEventSender) {
        egui::SidePanel::left("side_panel")
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.heading("Genjutsu");
                ui.separator();

                // === Model Info ===
                ui.heading(RichText::new("⚡ Shap-E").size(16.0));
                ui.add_space(5.0);

                ui.label(
                    RichText::new("OpenAI's fast text-to-3D model (~30-60 seconds)")
                        .small()
                        .color(Color32::LIGHT_BLUE)
                );

                ui.separator();

                // === Prompt Input ===
                ui.heading(RichText::new("✨ Text Prompt").size(16.0));
                ui.add_space(5.0);

                let text_edit = TextEdit::multiline(&mut self.prompt_text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("e.g., a red sports car, a medieval sword, a coffee mug...");

                ui.add(text_edit);

                ui.add_space(8.0);

                let generate_button = ui.add_enabled(
                    !self.is_generating && !self.prompt_text.trim().is_empty(),
                    egui::Button::new(
                        RichText::new("🎨 Generate 3D Model")
                            .size(14.0)
                    )
                        .min_size(egui::vec2(ui.available_width(), 30.0))
                );

                if generate_button.clicked() {
                    sender.instant(UiEvent::GenerateWithModel {
                        prompt: self.prompt_text.clone(),
                        model: self.selected_model,
                    });
                    self.is_generating = true;
                }

                ui.add_space(5.0);

                ui.separator();

                // === Example Prompts ===
                ui.collapsing("💡 Example Prompts", |ui| {
                    let examples = vec![
                        "a red sports car",
                        "a medieval sword",
                        "a blue crystal gem",
                        "a wooden chair",
                        "a futuristic robot",
                        "a coffee mug",
                        "a potted plant",
                        "a castle tower",
                        "a treasure chest",
                        "a flying drone",
                    ];

                    for example in examples {
                        if ui.button(example).clicked() {
                            self.prompt_text = example.to_string();
                        }
                    }
                });

                ui.separator();

                // === Tips ===
                ui.collapsing("💭 Prompt Tips", |ui| {
                    ui.label("✓ Be specific but simple");
                    ui.label("✓ Describe one object at a time");
                    ui.label("✓ Include colors and materials");
                    ui.label("✗ Avoid complex scenes");
                    ui.label("✗ Don't use abstract concepts");

                    ui.add_space(5.0);
                    ui.label(RichText::new("Examples:").strong());
                    ui.label("  Good: 'a red metal toolbox'");
                    ui.label("  Bad: 'happiness and joy'");
                });

                ui.separator();

                // === Status Display ===
                if let Some(ref s) = self.last_status {
                    let status_color = if s.contains("Error") || s.contains("Failed") {
                        Color32::from_rgb(255, 100, 100)
                    } else if s.contains("Generated") || s.contains("ready") || s.contains("success") {
                        Color32::from_rgb(100, 255, 100)
                    } else {
                        Color32::LIGHT_BLUE
                    };

                    ui.label(
                        RichText::new(format!("Status: {}", s))
                            .color(status_color)
                    );
                }

                ui.separator();

                // === Camera Controls ===
                ui.heading("🎮 Camera Controls");
                ui.label("• Left drag: Rotate");
                ui.label("• Mouse wheel: Zoom");

                if ui.button("🔄 Reset Camera").clicked() {
                    sender.instant(UiEvent::ResetCamera);
                }

                ui.separator();

                // === System Info ===
                ui.collapsing("ℹ️ System Info", |ui| {
                    ui.label("Model: Shap-E (OpenAI)");
                    ui.label("Renderer: Gaussian Splatting");
                    ui.label("Backend: WebGPU (wgpu)");
                    ui.label("Generation: ~30-60 seconds");
                });
            });
    }

    pub fn on_app_event(&mut self, ev: &AppEvent) {
        match ev {
            AppEvent::Status(s) => {
                self.last_status = Some(s.clone());

                if s.contains("Generated") || s.contains("Error") || s.contains("Failed") ||
                    s.contains("ready") || s.contains("success") || s.contains("Loaded") {
                    self.is_generating = false;
                }
            }
            AppEvent::Progress(p) => {
                self.last_status = Some(format!("Progress: {:.0}%", p * 100.0));
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