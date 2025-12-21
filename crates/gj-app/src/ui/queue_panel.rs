use std::sync::Arc;
use egui::{Color32, Context, RichText, Ui};
use crate::db::job::JobRecord;
use crate::events::{AppEvent, UiEvent};
use crate::job::JobStatus;
use crate::ui::{UiComponent, UiContext, UiEventSender};

#[derive(Default)]
pub struct QueuePanel {
    show_panel: bool,
    show_completed: bool,
}

impl QueuePanel {
    fn show_job_card(&self, ui: &mut Ui, job: &JobRecord, sender: &mut UiEventSender) {
        egui::Frame::none()
            .fill(Color32::from_gray(30))
            .rounding(5.0)
            .inner_margin(10.0)
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(60)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Status icon
                    ui.label(
                        RichText::new(job.status.icon())
                            .size(24.0)
                            .color(job.status.color())
                    );

                    ui.add_space(5.0);

                    // Job details
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&job.prompt).strong());
                            ui.label(
                                RichText::new(format!("({})", job.model))
                                    .small()
                                    .color(Color32::GRAY)
                            );
                        });

                        if let Some(message) = &job.message {
                            ui.label(
                                RichText::new(message)
                                    .small()
                                    .color(job.status.color())
                            );
                        }

                        // Time info
                        let created_date: chrono::DateTime<chrono::Utc> = job.created_at.clone().into();
                        let time_str = if let Some(completed) = &job.completed_at {
                            let completed_date: chrono::DateTime<chrono::Utc> = completed.to_utc().into();
                            let duration = (completed_date - created_date).num_seconds();
                            format!("Completed in {}s", duration)
                        } else {
                            let now = chrono::Utc::now();
                            format!("Elapsed: {}s", (now - created_date).num_seconds())
                        };
                        ui.label(RichText::new(time_str).small().color(Color32::GRAY));
                    });

                    // Right side - progress/actions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match &job.status {
                            JobStatus::Generating => {
                                ui.add(
                                    egui::ProgressBar::new(job.progress)
                                        .desired_width(150.0)
                                        .show_percentage()
                                        .animate(true)
                                );
                            }
                            JobStatus::Complete => {
                                // Show if scene is currently loaded
                                let is_current = job.ply_path.as_ref()
                                    .map(|p| p.to_string_lossy().contains(&job.job_id))
                                    .unwrap_or(false);

                                if is_current {
                                    ui.label(
                                        RichText::new("👁 Viewing")
                                            .color(Color32::LIGHT_BLUE)
                                    );
                                } else {
                                    if ui.button("📦 Load Scene").clicked() {
                                        sender.instant(UiEvent::LoadJobResult(job.job_id.clone()));
                                    }
                                }
                                ui.add_space(5.0);
                                if ui.button("🗑").clicked() {
                                    sender.instant(UiEvent::RemoveJob(job.job_id.clone()));
                                }
                            }
                            JobStatus::Failed => {
                                ui.label(
                                    RichText::new("Error")
                                        .color(Color32::RED)
                                        .small()
                                );
                                ui.add_space(5.0);
                                if ui.button("🗑").clicked() {
                                    sender.instant(UiEvent::RemoveJob(job.job_id.clone()));
                                }
                            }
                            JobStatus::Queued => {
                                ui.label(RichText::new("Waiting...").color(Color32::GRAY));
                            }
                            JobStatus::Submitting => {
                                ui.spinner();
                                ui.label(RichText::new("Submitting...").color(Color32::GRAY));
                            }
                        }
                    });
                });
            });

        ui.add_space(5.0);
    }
}

impl UiComponent for QueuePanel {
    fn show(&mut self, ctx: &Context, sender: &mut UiEventSender, ui_ctx: &UiContext) {
        if !self.show_panel {
            return;
        }

        egui::TopBottomPanel::bottom("queue_panel")
            .resizable(true)
            .min_height(100.0)
            .max_height(300.0)
            .default_height(150.0)
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.heading("🎬 Generation Queue");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Clear completed button
                        if ui.button("🗑 Clear Completed").clicked() {
                            sender.instant(UiEvent::ClearCompletedJobs);
                        }

                        ui.add_space(10.0);

                        // Toggle completed visibility
                        let toggle_text = if self.show_completed {
                            "Hide Completed"
                        } else {
                            "Show Completed"
                        };
                        if ui.button(toggle_text).clicked() {
                            self.show_completed = !self.show_completed;
                        }

                        ui.add_space(10.0);

                        // Stats
                        let active = ui_ctx.jobs.iter().filter(|j| j.status.is_active()).count();
                        let completed = ui_ctx.jobs.iter().filter(|j| j.status.is_complete()).count();

                        ui.label(
                            RichText::new(format!("Active: {} | Completed: {}", active, completed))
                                .color(Color32::GRAY)
                        );
                    });
                });

                ui.separator();

                // Job list
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut has_visible_jobs = false;

                        for job in &ui_ctx.jobs {
                            // Filter completed if hidden
                            if !self.show_completed && job.status.is_complete() {
                                continue;
                            }

                            has_visible_jobs = true;
                            self.show_job_card(ui, &job, sender);
                        }

                        if !has_visible_jobs {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("No jobs in queue")
                                        .color(Color32::GRAY)
                                        .size(16.0)
                                );
                            });
                        }
                    });
            });
    }

    fn on_app_event(&mut self, e: &AppEvent) {
        if let AppEvent::JobQueued(job) = e {

        }
    }
}