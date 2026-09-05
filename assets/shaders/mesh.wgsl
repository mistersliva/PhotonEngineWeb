// Mesh vertex + fragment shader for PhotonEngine wgpu port.
// Implements forward PBR-ish lighting with sun directional light,
// up to 12 point lights, flashlight, and shadow mapping.

// ─── Bindings ───

// Set 0: Uniform buffer (camera + lighting)
struct LightInfo {
    count: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    view_pos: vec4<f32>,
    ambient_sky: vec4<f32>,
    ambient_ground: vec4<f32>,
    light_vp: mat4x4<f32>,
    flash_pos: vec4<f32>,
    flash_dir: vec4<f32>,
    flash_params: vec4<f32>,
    light_info: LightInfo,
    lights_pos: array<vec4<f32>, 12>,
    lights_color: array<vec4<f32>, 12>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Set 1: Mesh texture
@group(1) @binding(0)
var mesh_texture: texture_2d<f32>;
@group(1) @binding(1)
var mesh_sampler: sampler;

// Set 2: Shadow map
@group(2) @binding(0)
var shadow_texture: texture_depth_2d;
@group(2) @binding(1)
var shadow_sampler: sampler_comparison;

// Set 3: Normal map
@group(3) @binding(0)
var normal_texture: texture_2d<f32>;
@group(3) @binding(1)
var normal_sampler: sampler;

// ─── Push constants (via uniform buffer update) ───
struct PushConstants {
    model: mat4x4<f32>,
    color: vec4<f32>,
    material: vec4<f32>, // metallic, roughness, ao, emissive
};

// Push constants are passed via a separate small uniform buffer
@group(4) @binding(0)
var<uniform> push: PushConstants;

// ─── Vertex stage ───

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) shadow_coord: vec4<f32>,
    @location(4) color: vec4<f32>,
    @location(5) material: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = push.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_pos = world_pos.xyz;
    // Transform normal by inverse transpose of model (upper-left 3x3)
    out.normal = normalize((push.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    out.shadow_coord = uniforms.light_vp * world_pos;
    out.color = push.color;
    out.material = push.material;
    return out;
}

// ─── Fragment stage ───

fn sample_shadow(coord: vec3<f32>) -> f32 {
    // Perspective divide
    let proj_coord = coord.xyz / coord.w;
    // Map from [-1,1] to [0,1] (wgpu clip space)
    let uv = vec2<f32>(proj_coord.x * 0.5 + 0.5, 0.5 - proj_coord.y * 0.5);
    let depth = proj_coord.z;

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth < 0.0 || depth > 1.0) {
        return 1.0;
    }

    // PCF 3x3
    var shadow = 0.0;
    let texel_size = 1.0 / 2048.0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompare(shadow_texture, shadow_sampler, uv.xy + offset, depth - 0.005);
        }
    }
    return shadow / 9.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(mesh_texture, mesh_sampler, in.uv).rgb * in.color.rgb;
    let metallic = in.material.x;
    let roughness = in.material.y;
    let ao = in.material.z;
    let emissive强度 = in.material.w;

    let N = normalize(in.normal);
    let V = normalize(uniforms.view_pos.xyz - in.world_pos);
    let L_sun = normalize(uniforms.light_dir.xyz);

    // Sun directional light (Lambert + simple specular)
    let NdotL_sun = max(dot(N, L_sun), 0.0);
    let H_sun = normalize(L_sun + V);
    let NdotH_sun = max(dot(N, H_sun), 0.0);
    let spec_sun = pow(NdotH_sun, mix(8.0, 128.0, 1.0 - roughness)) * (1.0 - roughness);
    let sun_diffuse = uniforms.light_color.rgb * NdotL_sun;
    let sun_spec = uniforms.light_color.rgb * spec_sun * (metallic * 0.8 + 0.2);

    // Shadow
    let shadow = sample_shadow(in.shadow_coord.xyz);
    let sun_contrib = (sun_diffuse + sun_spec) * shadow;

    // Ambient (ground + sky hemispheric)
    let up_factor = dot(N, vec3<f32>(0.0, 1.0, 0.0)) * 0.5 + 0.5;
    let ambient = mix(uniforms.ambient_ground.rgb, uniforms.ambient_sky.rgb, up_factor) * ao;

    // Point lights (HL2 style)
    var point_contrib = vec3<f32>(0.0);
    let light_count = i32(uniforms.light_info.count);
    for (var i = 0; i < 12; i++) {
        if (i >= light_count) { break; }
        let light_pos = uniforms.lights_pos[i].xyz;
        let light_radius = uniforms.lights_pos[i].w;
        let light_color = uniforms.lights_color[i].rgb;
        let light_intensity = uniforms.lights_color[i].w;

        let to_light = light_pos - in.world_pos;
        let dist = length(to_light);
        if (dist > light_radius) { continue; }

        let L = to_light / dist;
        let NdotL = max(dot(N, L), 0.0);
        // Smooth attenuation (HL2 style squared falloff)
        let atten = 1.0 - saturate(dist / light_radius);
        let atten_sq = atten * atten;
        point_contrib += light_color * light_intensity * NdotL * atten_sq;
    }

    // Flashlight
    var flash_contrib = vec3<f32>(0.0);
    let flash_active = uniforms.flash_pos.w;
    if (flash_active > 0.5) {
        let to_flash = uniforms.flash_pos.xyz - in.world_pos;
        let flash_dist = length(to_flash);
        let flash_range = uniforms.flash_params.z;
        if (flash_dist < flash_range) {
            let L_flash = to_flash / flash_dist;
            let inner_cos = uniforms.flash_params.x;
            let outer_cos = uniforms.flash_params.y;
            let spot_cos = dot(-L_flash, normalize(uniforms.flash_dir.xyz));
            let spot_atten = saturate((spot_cos - outer_cos) / (inner_cos - outer_cos));
            let dist_atten = 1.0 - saturate(flash_dist / flash_range);
            let flash_int = uniforms.flash_params.w;
            let NdotL_flash = max(dot(N, L_flash), 0.0);
            flash_contrib = vec3<f32>(1.0, 0.95, 0.85) * flash_int * NdotL_flash * spot_atten * dist_atten * dist_atten;
        }
    }

    // Emissive
    let emissive = albedo * emissive强度;

    let color = albedo * (ambient + sun_contrib + point_contrib + flash_contrib) + emissive;

    // Simple tonemap
    let mapped = color / (color + vec3<f32>(1.0));
    // Gamma correction
    let final_color = pow(mapped, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(final_color, 1.0);
}
