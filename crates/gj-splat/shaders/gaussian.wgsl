// Simplified Gaussian Splatting Shader - Performance Optimized

struct Uniforms {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding1: f32,
    viewport: vec2<f32>,
    focal: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) _padding1: f32,
    @location(3) color: vec3<f32>,
    @location(4) opacity: f32,
    @location(5) scale: vec3<f32>,
    @location(6) _padding2: f32,
    @location(7) rotation: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) opacity: f32,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Project center
    let clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);

    // Use actual scale from data (much larger multiplier)
    let avg_scale = (in.scale.x + in.scale.y + in.scale.z) / 3.0;
    let radius = avg_scale * 500.0; // Increased from 100.0 to 500.0

    // Create billboard quad
    let view_space_pos = uniforms.view * vec4<f32>(in.position, 1.0);
    let distance_factor = max(-view_space_pos.z, 0.1);
    let screen_radius = radius / distance_factor;

    // Expand quad (adjust for aspect ratio)
    let screen_offset = in.quad_pos * screen_radius * vec2<f32>(uniforms.viewport.y / uniforms.viewport.x, 1.0);
    let ndc_offset = screen_offset / uniforms.viewport * 2.0;

    out.clip_position = vec4<f32>(
        clip_pos.x / clip_pos.w + ndc_offset.x,
        clip_pos.y / clip_pos.w + ndc_offset.y,
        clip_pos.z / clip_pos.w,
        1.0
    );

    out.color = in.color;
    out.opacity = in.opacity;
    out.uv = in.quad_pos;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Softer gaussian falloff
    let dist = length(in.uv);

    // Discard outside circle
    if dist > 1.0 {
        discard;
    }

    // Softer falloff (reduced exponent for larger visible area)
    let alpha = exp(-dist * dist * 1.0) * in.opacity;  // Changed from 2.0 to 1.0

    if alpha < 0.005 {  // Lower threshold
        discard;
    }

    return vec4<f32>(in.color * alpha, alpha);
}