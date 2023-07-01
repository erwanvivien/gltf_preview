#[cfg(not(target_arch = "wasm32"))]
fn init_log() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("mario_skurt", log::LevelFilter::Info)
        .format_timestamp(None)
        .init();
}

fn main() -> Result<(), ()> {
    #[cfg(not(target_arch = "wasm32"))]
    init_log();

    let mut scenes = mario_skurt::load_scenes("assets/CesiumMilkTruck.glb")?;
    pollster::block_on(mario_skurt::run(&mut scenes));

    Ok(())
}
