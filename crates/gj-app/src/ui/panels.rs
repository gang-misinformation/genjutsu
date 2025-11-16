use egui::Context;
use crate::events::AppEvent;
use crate::ui::panels::central_panel::CentralPanel;
use crate::ui::panels::side_panel::SidePanel;
use crate::ui::panels::top_panel::TopPanel;
use crate::ui::UiEventSender;

mod top_panel;
mod side_panel;
mod central_panel;

pub struct Panels {
    pub top: TopPanel,
    pub side: SidePanel,
    pub central: CentralPanel,
}

impl Default for Panels {
    fn default() -> Self {
        Self {
            top: TopPanel::default(),
            side: SidePanel::default(),
            central: CentralPanel::default(),
        }
    }
}

impl Panels {
    /// Draw all panels. Each panel can push UiEvents into the sender.
    pub fn draw(&mut self, ctx: &Context, sender: &mut UiEventSender) {
        self.top.show(ctx, sender);
        self.side.show(ctx, sender);
        self.central.show(ctx, sender);
    }

    /// Broadcast AppEvent to each panel (child components can react)
    pub fn on_app_event(&mut self, ev: &AppEvent) {
        self.top.on_app_event(ev);
        self.side.on_app_event(ev);
        self.central.on_app_event(ev);
    }
}