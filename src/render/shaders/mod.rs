use std::{collections::HashMap, rc::Rc};

use crate::utils::load_file_string;

static mut GLOBAL_SHADERS: Option<HashMap<String, Rc<wgpu::ShaderModule>>> = None;

const SHADERS: [&str; 2] = ["albedo_shader", "texture_shader"];

pub async fn build_shaders(device: &wgpu::Device) {
    for shader_name in &SHADERS {
        get_shader_or_build(shader_name, device).await;
    }
}

pub fn get_shader(name: &str) -> Rc<wgpu::ShaderModule> {
    #[cfg(feature = "debug_shader")]
    log::info!("Getting shader {:?}", name);

    unsafe { GLOBAL_SHADERS.as_ref() }.expect("Shaders not built")[name].clone()
}

async fn get_shader_or_build(name: &str, device: &wgpu::Device) -> Rc<wgpu::ShaderModule> {
    if unsafe { GLOBAL_SHADERS.as_ref() }.is_none() {
        #[cfg(feature = "debug_shader")]
        log::info!("Init shader storage");
        unsafe { GLOBAL_SHADERS = Some(HashMap::new()) };
    }

    let global_shaders = unsafe { GLOBAL_SHADERS.as_mut() };
    if global_shaders.as_ref().unwrap().contains_key(name) {
        #[cfg(feature = "debug_shader")]
        log::info!("Using cached shader {:?}", name);
        return unsafe { GLOBAL_SHADERS.as_ref() }.unwrap()[name].clone();
    }

    #[cfg(feature = "debug_shader")]
    log::info!("Building shader {:?}", name);

    let shader = load_file_string(format!("assets/{name}.wgsl"))
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

    shader
}
