use env_logger::Env;

mod engine;
mod renderer;
mod state;
mod util;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("goldenrod", log::LevelFilter::Info)
        .init();

    engine::run();
}
