// Vertex shader

const NONE: u32 = 0u;
const POSITION: u32 = 0x01u;
const NORMAL: u32 = 0x02u;
const TEX_COORD_0: u32 = 0x04u;
const TEX_COORD_1: u32 = 0x08u;
const TANGENT: u32 = 0x10u;
const WEIGHT: u32 = 0x20u;
const JOINT: u32 = 0x40u;
const COLOR: u32 = 0x80u;

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
    @location(10) shader_kinds: u32,
};

struct InstanceInput {
    @location(11) transform_column_0: vec4<f32>,
    @location(12) transform_column_1: vec4<f32>,
    @location(13) transform_column_2: vec4<f32>,
    @location(14) transform_column_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) shader_kinds: u32,
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
    out.tex_coords = model.tex_coords_0;
    out.color = model.color;
    out.shader_kinds = model.shader_kinds;
    out.clip_position = camera * transform * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (in.shader_kinds & COLOR) != 0u {
        return in.color;
    }
    return texture_sample;
}
