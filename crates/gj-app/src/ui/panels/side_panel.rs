use egui::{Context, RichText, TextEdit, ComboBox, Color32};
use gj_core::Model3D;
use crate::events::{AppEvent, UiEvent};
use crate::ui::UiEventSender;

pub struct SidePanel {
    // Model selection
    pub selected_model: Model3D,

    // Render settings
    pub show_grid: bool,
    pub last_status: Option<String>,

    // Prompt input
    pub prompt_text: String,
    pub is_generating: bool,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            selected_model: Model3D::DreamScene360,
            show_grid: false,
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

                // === Model Selection ===
                ui.heading(RichText::new("🎯 Generation Model").size(16.0));
                ui.add_space(5.0);

                ComboBox::from_label("Select Model")
                    .selected_text(format!("{} {}",
                                           self.selected_model.icon(),
                                           self.selected_model.name()
                    ))
                    .show_ui(ui, |ui| {
                        for model in Model3D::all() {
                            let text = format!("{} {} - {}",
                                               model.icon(),
                                               model.name(),
                                               model.description()
                            );

                            ui.selectable_value(
                                &mut self.selected_model,
                                model,
                                text
                            );
                        }
                    });

                ui.add_space(3.0);
                ui.label(
                    RichText::new(self.selected_model.description())
                        .small()
                        .color(Color32::LIGHT_BLUE)
                );

                ui.separator();

                // === Prompt Input ===
                ui.heading(RichText::new("✨ Text Prompt").size(16.0));
                ui.add_space(5.0);

                let prompt_hint = match self.selected_model {
                    Model3D::DreamScene360 => "e.g., a medieval castle courtyard, a futuristic cityscape...",
                    Model3D::GaussianDreamerPro => "e.g., a red sports car, a marble statue...",
                    Model3D::TripoSR => "e.g., a coffee mug, a chair...",
                    Model3D::SceneScape => "e.g., a living room with sofa and TV, a workshop...",
                };

                let text_edit = TextEdit::multiline(&mut self.prompt_text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text(prompt_hint);

                ui.add(text_edit);

                ui.add_space(8.0);

                let generate_button = ui.add_enabled(
                    !self.is_generating && !self.prompt_text.trim().is_empty(),
                    egui::Button::new(
                        RichText::new(format!("🎨 Generate with {}", self.selected_model.name()))
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

                // Show placeholder warning
                ui.label(
                    RichText::new("⚠️ Models not installed yet - using placeholder generation")
                        .small()
                        .color(Color32::from_rgb(255, 200, 100))
                );

                ui.separator();

                // === Example Prompts by Model ===
                ui.collapsing("💡 Example Prompts", |ui| {
                    let examples = match self.selected_model {
                        Model3D::DreamScene360 => vec![
                            "a cozy forest clearing with campfire",
                            "a cyberpunk street at night",
                            "a medieval throne room",
                            "a tropical beach with palm trees",
                        ],
                        Model3D::GaussianDreamerPro => vec![
                            "a red sports car",
                            "a blue crystal gem",
                            "a wooden chair",
                            "a golden trophy",
                        ],
                        Model3D::TripoSR => vec![
                            "a coffee mug",
                            "a house plant",
                            "a toy robot",
                            "a lamp",
                        ],
                        Model3D::SceneScape => vec![
                            "a modern kitchen with island",
                            "a home office with desk and bookshelf",
                            "a bedroom with bed and nightstand",
                            "a garage workshop with tools",
                        ],
                    };

                    for example in examples {
                        if ui.button(example).clicked() {
                            self.prompt_text = example.to_string();
                        }
                    }
                });

                ui.separator();

                // === Alternative: Load from Files ===
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
                ui.heading("Camera Controls");
                ui.label("• Left drag: Rotate");
                ui.label("• Mouse wheel: Zoom");

                if ui.button("🔄 Reset Camera").clicked() {
                    sender.instant(UiEvent::ResetCamera);
                }
            });
    }

    pub fn on_app_event(&mut self, ev: &AppEvent) {
        match ev {
            AppEvent::Status(s) => {
                self.last_status = Some(s.clone());

                if s.contains("Generated") || s.contains("Error") || s.contains("Failed") || s.contains("ready") || s.contains("success") {
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