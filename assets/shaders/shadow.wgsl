// Shadow map vertex shader - depth-only pass from sun's perspective.

struct ShadowPushConstants {
    model: mat4x4<f32>,
    light_vp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> pc: ShadowPushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = pc.model * vec4<f32>(in.position, 1.0);
    out.clip_position = pc.light_vp * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Depth-only - output is written to depth buffer automatically.
    return vec4<f32>(0.0);
}
