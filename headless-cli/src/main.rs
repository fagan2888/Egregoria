use argh::FromArgs;
use common::TimeInfo;
use egregoria::Egregoria;
use log::LevelFilter;
use std::path::Path;

#[derive(FromArgs)]
#[argh(description = "\
Egregoria's headless cli for running egregoria scenarios.\n\
Example: goria test.lua")]
struct Args {
    #[argh(positional)]
    scenario: Vec<String>,
}

fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .filter(Some("egregoria"), LevelFilter::Warn)
        .init();

    let args: Args = argh::from_env();

    for scenario in args.scenario {
        if let Ok(r) = std::fs::read_dir(&scenario) {
            for p in r.filter_map(|x| x.ok()) {
                let state = egregoria::Egregoria::init();
                run(state, p.path().as_path())
            }
        } else {
            let state = egregoria::Egregoria::init();
            run(state, scenario.as_str().as_ref())
        }
    }
}

fn run(mut state: Egregoria, name: &Path) {
    let l = match mods::load(name) {
        Some(l) => l,
        None => {
            return;
        }
    };

    egregoria::scenarios::add_egregoria_lua_stdlib(&l, &mut state);
    mods::eval_f(&l, "Init");

    for i in 1..1000 {
        step(&mut state);

        let v: Option<bool> = mods::call_f(&l, "Success");
        let v = match v {
            Some(x) => x,
            None => {
                return;
            }
        };

        if v {
            log::info!("success for {:?} at iteration {}", name, i);
            return;
        }
    }

    log::warn!("failure for {:?}", name);
}

const TIME_STEP: f64 = 1.0 / 30.0;

fn step(state: &mut Egregoria) {
    {
        let mut time = state.write::<TimeInfo>();
        time.delta = TIME_STEP as f32;
        time.time_speed = 1.0;
        time.time += TIME_STEP;
        time.time_seconds = time.time as u64;
    }
    state.run();
}
