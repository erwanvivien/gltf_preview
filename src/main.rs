#[cfg(not(target_arch = "wasm32"))]
fn init_log() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("mario_skurt", log::LevelFilter::Info)
        .format_target(false)
        .init();
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    init_log();

    pollster::block_on(mario_skurt::run());
}
