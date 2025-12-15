#![recursion_limit = "256"]

mod app;
mod state;
mod ui;
mod events;
mod gfx;
mod generator;
mod error;

use std::error::Error;
use winit::event_loop::{ControlFlow, EventLoop};
use crate::events::GjEvent;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut event_loop: EventLoop<GjEvent> = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}