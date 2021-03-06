#![allow(clippy::too_many_arguments)]

use crate::context::Context;
use log::{Level, LevelFilter};
use std::io::Write;
use std::time::Instant;

mod audio;
mod context;
mod game_loop;
mod gui;
mod input;
mod rendering;

fn main() {
    let start = Instant::now();
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .filter(Some("wgpu_core"), LevelFilter::Warn)
        .filter(Some("gfx_memory"), LevelFilter::Warn)
        .filter(Some("gfx_backend_vulkan"), LevelFilter::Warn)
        .filter(Some("gfx_descriptor"), LevelFilter::Warn)
        .format(move |f, r| {
            if std::thread::panicking() {
                return Ok(());
            }
            let time = Instant::now().duration_since(start).as_micros();
            if r.level() > Level::Warn {
                let module_path = r
                    .module_path_static()
                    .and_then(|x| x.split(':').last())
                    .unwrap_or_default();
                writeln!(
                    f,
                    "[{:9} {:5} {:12}] {}",
                    time,
                    r.metadata().level().to_string(),
                    module_path,
                    r.args()
                )
            } else {
                writeln!(
                    f,
                    "[{:9} {:5} {}:{}] {}",
                    time,
                    r.metadata().level().to_string(),
                    r.file().unwrap_or_default(),
                    r.line().unwrap_or_default(),
                    r.args()
                )
            }
        })
        .init();
    let mut ctx = Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
