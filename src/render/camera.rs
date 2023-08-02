use winit::window::Window;

#[derive(Debug, Clone, Copy)]
enum Angle {
    Radians(f32),
    Degrees(f32),
}

impl Angle {
    fn to_radians(&self) -> f32 {
        match self {
            Angle::Radians(radians) => *radians,
            Angle::Degrees(degrees) => degrees.to_radians(),
        }
    }
}

pub struct Camera {
    eye: glam::Vec3,
    // Horizontal angle
    yaw: Angle,
    // Vertical angle
    pitch: Angle,

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
        use wgpu::util::DeviceExt;

        let inner_size = window.inner_size();

        let eye = glam::vec3(0.0, 0.0, 0.0);
        let yaw = Angle::Degrees(0f32);
        let pitch = Angle::Degrees(0f32);

        let aspect = inner_size.width as f32 / inner_size.height as f32;
        let fovy = 45.0 / 180.0 * std::f32::consts::PI;
        let znear = 0.1;
        let zfar = 10000.0;

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
            yaw,
            pitch,

            aspect,
            fovy,
            znear,
            zfar,

            buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.to_radians().sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.to_radians().sin_cos();

        let forward = glam::Vec3::new(cos_yaw * cos_pitch, sin_pitch, sin_yaw * cos_pitch);

        let view = glam::Mat4::look_to_lh(self.eye, forward.normalize(), glam::Vec3::Y);
        let projection = glam::Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);

        projection * view
    }

    pub fn update_projection_matrix(&self, queue: &wgpu::Queue) {
        let projection_matrix = self.projection_matrix();

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[projection_matrix]));
    }

    pub fn move_camera<P: Into<glam::Vec3>>(&mut self, direction: P) {
        // Update direction to be relative to forward
        // We are using *_lh functions, so we need to negate the yaw
        let direction_forward =
            glam::Quat::from_rotation_y(-self.yaw.to_radians()).mul_vec3(direction.into());

        self.set_camera(self.eye + direction_forward);
    }

    pub fn set_camera<P: Into<glam::Vec3>>(&mut self, position: P) {
        self.eye = position.into();
    }

    pub fn move_yaw_pitch(&mut self, yaw: f32, pitch: f32) {
        const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - f32::EPSILON;

        // If yaw and pitch are both 0, we don't need to do anything
        if yaw.abs() < f32::EPSILON && pitch.abs() < f32::EPSILON {
            return;
        }

        // Yaw is negative because we are using *_lh functions
        self.yaw = Angle::Radians(self.yaw.to_radians() - yaw);
        // Pitch is always negative
        let pitch = (self.pitch.to_radians() - pitch).clamp(-MAX_PITCH, MAX_PITCH);
        self.pitch = Angle::Radians(pitch);
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }
}
