use std::{collections::HashMap, path::Path, rc::Rc};

static mut GLOBAL_SHADERS: Option<HashMap<String, Rc<wgpu::ShaderModule>>> = None;

fn get_current_dir() -> &'static Path {
    Path::new(file!()).parent().unwrap()
}

pub fn build_shaders(device: &wgpu::Device) {
    let mut shaders = HashMap::new();

    for file in std::fs::read_dir(get_current_dir()).expect("Could not read dir") {
        let file = file.expect("Could not read file");
        let path = file.path();

        if path.extension() == Some("wgsl".as_ref()) {
            #[cfg(feature = "debug_all")]
            log::info!("Loading shader {:?}", &path);

            let shader = std::fs::read_to_string(&path).expect("Could not read shader");
            let name = path.file_stem().unwrap().to_str().unwrap();

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(shader.into()),
            };

            let shader = device.create_shader_module(shader);

            shaders.insert(String::from(name), Rc::new(shader));
        }
    }

    unsafe {
        GLOBAL_SHADERS = Some(shaders);
    }
}

pub fn get_shader(name: &str) -> Rc<wgpu::ShaderModule> {
    #[cfg(feature = "debug_all")]
    log::info!("Getting shader {:?}", name);
    unsafe { GLOBAL_SHADERS.as_ref() }.expect("Shaders not built")[name].clone()
}

pub fn _get_shader_or_build(name: &str, device: &wgpu::Device) -> Rc<wgpu::ShaderModule> {
    let path = get_current_dir().join(format!("{}.wgsl", name));
    assert!(path.exists(), "Shader {:?} does not exist", path);

    let shader = std::fs::read_to_string(&path).expect("Could not read shader");
    let name = path.file_stem().unwrap().to_str().unwrap();

    let shader = wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    };

    let shader = device.create_shader_module(shader);
    let shader = Rc::new(shader);

    unsafe { GLOBAL_SHADERS.as_mut() }
        .unwrap()
        .insert(String::from(name), shader.clone());

    shader
}
