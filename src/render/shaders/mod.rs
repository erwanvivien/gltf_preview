use std::{collections::HashMap, rc::Rc};

use crate::utils::load_file_string;

pub mod kind;

static mut GLOBAL_SHADERS: Option<HashMap<String, Rc<wgpu::ShaderModule>>> = None;

const SHADERS: [&str; 1] = ["main_shader"];

pub async fn build_shaders(device: &wgpu::Device) {
    for shader_name in &SHADERS {
        build_shader(device, shader_name).await;
    }
}

pub fn get_shader(name: &str) -> Rc<wgpu::ShaderModule> {
    #[cfg(feature = "debug_shader")]
    log::info!("Getting shader {:?}", name);

    unsafe { GLOBAL_SHADERS.as_ref() }.expect("No shaders built")[name].clone()
}

async fn build_shader(device: &wgpu::Device, name: &str) {
    if unsafe { GLOBAL_SHADERS.as_ref() }.is_none() {
        #[cfg(feature = "debug_shader")]
        log::info!("Init shader storage");
        unsafe { GLOBAL_SHADERS = Some(HashMap::new()) };
    }

    let global_shaders = unsafe { GLOBAL_SHADERS.as_mut() };
    if global_shaders.as_ref().unwrap().contains_key(name) {
        #[cfg(feature = "debug_shader")]
        log::info!("Shader already built: {}", name);
        return;
    }

    #[cfg(feature = "debug_shader")]
    log::info!("Building shader {:?}", name);

    let shader: String = load_file_string(format!("assets/{name}.wgsl"))
        .await
        .expect("Could not read shader");

    let shader = wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    };

    let shader = device.create_shader_module(shader);
    let shader = Rc::new(shader);

    global_shaders
        .unwrap()
        .insert(String::from(name), shader.clone());
}
