fn main() -> Result<(), ()> {
    let mut scenes = mario_skurt::model::load_scenes("assets/Fox.glb")?;
    let first_scene = scenes.pop().unwrap();

    pollster::block_on(mario_skurt::run(first_scene));

    Ok(())
}
