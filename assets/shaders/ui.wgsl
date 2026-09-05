// UI quad vertex + fragment shader.
// Draws textured, colored 2D quads for the HUD and menu overlay.

struct UiUniforms {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: UiUniforms;

@group(1) @binding(0)
var ui_texture: texture_2d<f32>;
@group(1) @binding(1)
var ui_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Map from [-aspect..aspect, -1..1] to NDC [0..1, 0..1] then to clip space
    // Assuming input is already in normalized device coordinates (-1 to 1)
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(ui_texture, ui_sampler, in.uv);
    return tex_color * in.color;
}
