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

    let mut scenes = mario_skurt::model::load_scenes("assets/Fox.glb")?;
    let first_scene_fox = scenes.pop().unwrap();

    pollster::block_on(mario_skurt::run(&mut [first_scene_fox]));

    Ok(())
}
