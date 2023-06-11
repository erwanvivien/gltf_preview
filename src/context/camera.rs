use winit::window::Window;

pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
}

const CAMERA_BUFFER_LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
};

impl Camera {
    pub fn new(window: &Window, device: &wgpu::Device) -> Self {
        let inner_size = window.inner_size();

        let eye = glam::vec3(0.0, 0.0, 300.0);
        let target = glam::vec3(0.0, 0.0, 0.0);
        let up = glam::Vec3::Y;
        let aspect = inner_size.width as f32 / inner_size.height as f32;
        let fovy = 45.0 / 180.0 * std::f32::consts::PI;
        let znear = 0.1;
        let zfar = 10000.0;

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[glam::Mat4::IDENTITY]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[CAMERA_BUFFER_LAYOUT],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: Some("Camera bind group"),
        });

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,

            buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_lh(self.eye, self.target, self.up);
        let projection = glam::Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);

        projection * view
    }

    pub fn update_projection_matrix(&self, queue: &wgpu::Queue) {
        let projection_matrix = self.projection_matrix();

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[projection_matrix]));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }
}
