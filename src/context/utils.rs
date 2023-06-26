const TRANSFORM_GROUP_LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
};

pub fn get_or_create_transform_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
    static mut BIND_GROUP_LAYOUT: Option<wgpu::BindGroupLayout> = None;

    if unsafe { BIND_GROUP_LAYOUT.is_none() } {
        let bind_group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[TRANSFORM_GROUP_LAYOUT],
            label: Some("Global Transform Bind Group Layout"),
        });

        unsafe {
            BIND_GROUP_LAYOUT = Some(bind_group);
        }
    }

    unsafe { BIND_GROUP_LAYOUT.as_ref() }.unwrap()
}
