// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) _padding_0: f32,
    @location(2) normal: vec3<f32>,
    @location(3) _padding_1: f32,
    @location(4) tex_coords_0: vec2<f32>,
    @location(5) tex_coords_1: vec2<f32>,
    @location(6) tangents: vec4<f32>,
    @location(7) weights: vec4<f32>,
    @location(8) joints: vec4<u32>,
    @location(9) color: vec4<f32>,
};

struct InstanceInput {
    @location(10) transform_column_0: vec4<f32>,
    @location(11) transform_column_1: vec4<f32>,
    @location(12) transform_column_2: vec4<f32>,
    @location(13) transform_column_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) 
var<uniform> camera: mat4x4<f32>;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let transform = mat4x4<f32>(
        instance.transform_column_0,
        instance.transform_column_1,
        instance.transform_column_2,
        instance.transform_column_3,
    );

    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera * transform * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
