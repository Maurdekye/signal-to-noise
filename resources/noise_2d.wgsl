struct Uniforms {
    resolution: vec2<f32>,
    grid_spacing: f32,
    signal_origin: vec2<f32>,
    signal_strength: f32,
    signal_width: f32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
}

@group(3) @binding(0)
var<uniform> params: Uniforms;

fn smooth_clamp(x: f32, a: f32) -> f32 {
    let z = clamp(x / a, -1.5, 1.5);
    return a * (z - ((4.0/27.0) *z*z*z));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / params.resolution;
    let grid_coord = floor(uv / params.grid_spacing);
    let norm = min(params.resolution.x, params.resolution.y);
    
    let base_seed = dot(grid_coord, vec2<f32>(12.9898, 78.233));
    let seed = base_seed + params.noise_seed;

    let u1 = fract(sin(seed) * 43758.5453);
    let u2 = fract(sin(seed + 1.0) * 43758.5453);

    let z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * 3.14159265 * u2);
    let z0_clamped = smooth_clamp(z0, params.noise_deviation_cap);
    let noise_shade = params.noise_deviation * z0_clamped + params.noise_floor;

    let clamped_position = (grid_coord + 0.5) * params.grid_spacing;
    let signal_origin_dist = length(clamped_position*params.resolution - params.signal_origin) / norm;
    let factor = signal_origin_dist / params.signal_width;
    let signal_shade = exp(-factor*factor) * params.signal_strength;
    let shade = noise_shade + signal_shade;

    return vec4<f32>(shade, shade, shade, 1.0);
}