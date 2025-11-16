// Proper 3D Gaussian Splatting Shader with Instanced Rendering

struct Uniforms {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    camera_pos: vec3<f32>,
    viewport: vec2<f32>,
    focal: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,          // Quad corner position
    @location(1) position: vec3<f32>,          // Gaussian center
    @location(2) _padding1: f32,               // padding
    @location(3) color: vec3<f32>,
    @location(4) opacity: f32,
    @location(5) scale: vec3<f32>,
    @location(6) _padding2: f32,               // padding
    @location(7) rotation: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) opacity: f32,
    @location(2) conic: vec3<f32>,  // [A, B, C] for conic section
    @location(3) center: vec2<f32>,  // projected center
}

// Quaternion to rotation matrix
fn quat_to_mat3(q: vec4<f32>) -> mat3x3<f32> {
    let w = q.x;
    let x = q.y;
    let y = q.z;
    let z = q.w;
    
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    
    return mat3x3<f32>(
        vec3<f32>(1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)),
        vec3<f32>(2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)),
        vec3<f32>(2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy))
    );
}

// Compute 3D covariance matrix from scale and rotation
fn compute_covariance_3d(scale: vec3<f32>, rotation: vec4<f32>) -> mat3x3<f32> {
    let R = quat_to_mat3(rotation);
    let S = mat3x3<f32>(
        vec3<f32>(scale.x, 0.0, 0.0),
        vec3<f32>(0.0, scale.y, 0.0),
        vec3<f32>(0.0, 0.0, scale.z)
    );
    
    // Σ = R * S * S^T * R^T
    let RS = R * S;
    return RS * transpose(RS);
}

// Project 3D covariance to 2D screen space
fn project_covariance(
    mean: vec3<f32>,
    cov3d: mat3x3<f32>,
    view_mat: mat4x4<f32>,
    focal: vec2<f32>
) -> mat2x2<f32> {
    // Transform to view space
    let t = (view_mat * vec4<f32>(mean, 1.0)).xyz;
    
    // Avoid division by zero
    let tz = max(abs(t.z), 0.001);
    
    // Jacobian of perspective projection
    let limx = 1.3 * uniforms.viewport.x;
    let limy = 1.3 * uniforms.viewport.y;
    let txtz = t.x / tz;
    let tytz = t.y / tz;
    
    let J = mat3x2<f32>(
        vec2<f32>(focal.x / tz, 0.0),
        vec2<f32>(0.0, focal.y / tz),
        vec2<f32>(-focal.x * txtz / tz, -focal.y * tytz / tz)
    );
    
    // View matrix rotation part (3x3)
    let W = mat3x3<f32>(
        view_mat[0].xyz,
        view_mat[1].xyz,
        view_mat[2].xyz
    );
    
    // Transform covariance to view space
    let cov_view = W * cov3d * transpose(W);
    
    // Project to 2D: Σ' = J * Σ_view * J^T
    let cov2d = J * cov_view * transpose(J);
    
    // Add a small value to diagonal for numerical stability
    let stabilizer = 0.3;
    return mat2x2<f32>(
        vec2<f32>(cov2d[0][0] + stabilizer, cov2d[0][1]),
        vec2<f32>(cov2d[1][0], cov2d[1][1] + stabilizer)
    );
}

// Compute conic parameters (inverse covariance) for fragment shader
fn compute_conic(cov2d: mat2x2<f32>) -> vec3<f32> {
    let det = cov2d[0][0] * cov2d[1][1] - cov2d[0][1] * cov2d[1][0];
    
    if abs(det) < 1e-6 {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    
    let inv_det = 1.0 / det;
    
    // Inverse of 2x2 matrix
    return vec3<f32>(
        cov2d[1][1] * inv_det,  // A
        -cov2d[0][1] * inv_det, // B
        cov2d[0][0] * inv_det   // C
    );
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Compute 3D covariance
    let cov3d = compute_covariance_3d(in.scale, in.rotation);
    
    // Project to screen space
    let cov2d = project_covariance(
        in.position,
        cov3d,
        uniforms.view,
        uniforms.focal
    );
    
    // Compute conic for fragment shader
    out.conic = compute_conic(cov2d);
    
    // Project center
    let clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    let ndc = clip_pos.xy / clip_pos.w;
    out.center = (ndc * 0.5 + 0.5) * uniforms.viewport;

    // Calculate extent of splat (3 sigma)
    let mid = 0.5 * (cov2d[0][0] + cov2d[1][1]);
    let det = cov2d[0][0] * cov2d[1][1] - cov2d[0][1] * cov2d[0][1];
    let lambda1 = mid + sqrt(max(0.1, mid * mid - det));
    let lambda2 = mid - sqrt(max(0.1, mid * mid - det));
    let radius = ceil(3.0 * sqrt(max(lambda1, lambda2)));

    // Expand quad by radius
    let screen_pos = out.center + in.quad_pos * radius;

    // Convert back to clip space
    let ndc_pos = (screen_pos / uniforms.viewport) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc_pos.x, ndc_pos.y, clip_pos.z / clip_pos.w, 1.0);

    // Pass through
    out.color = in.color;
    out.opacity = in.opacity;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get fragment position in screen space
    let frag_coord = vec2<f32>(in.clip_position.x, in.clip_position.y);
    
    // Compute distance from center using conic (inverse covariance)
    let d = frag_coord - in.center;
    let power = -0.5 * (in.conic.x * d.x * d.x + 2.0 * in.conic.y * d.x * d.y + in.conic.z * d.y * d.y);
    
    // Clamp power to avoid overflow
    let clamped_power = clamp(power, -10.0, 0.0);
    
    // Gaussian falloff
    let alpha = exp(clamped_power);
    
    // Modulate by opacity
    let final_alpha = alpha * in.opacity;
    
    // Discard transparent pixels
    if final_alpha < 0.02 {
        discard;
    }
    
    // Premultiply alpha
    return vec4<f32>(in.color * final_alpha, final_alpha);
}