struct Uniforms {
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    grid_spacing: f32,
    shift_duration: f32,
    time: f32,
}

@group(3) @binding(0)
var<uniform> params: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / params.resolution;
    let grid_coord = floor(uv / params.grid_spacing);
    let time_frame = floor(params.time / params.shift_duration);
    
    let base_seed = dot(grid_coord, vec2<f32>(12.9898, 78.233));
    let seed = base_seed + time_frame;

    // Original uniform random value
    let u1 = fract(sin(seed) * 43758.5453);
    // A second independent uniform random value (offset the seed by a constant)
    let u2 = fract(sin(seed + 1.0) * 43758.5453);

    // Box–Muller transform: convert (u1, u2) to a standard normal value z0
    let z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * 3.14159265 * u2);
    // Clamp z0 to [-5, 5] (5 standard deviations)
    let z0_clamped = clamp(z0, -4.0, 4.0);
    // Scale and shift to get a normal distribution centered at 0.5 with stddev ≈ 0.15
    let noise_shade = 0.05 * z0_clamped + 0.25;

    let mouse_distance = length(position.xy - params.mouse) / min(params.resolution.x, params.resolution.y);
    let factor = mouse_distance * 4.0;
    let signal_shade = exp(-factor*factor) * 0.1;
    let shade = noise_shade + signal_shade;
    // let shade = signal_shade;

    return vec4<f32>(shade, shade, shade, 1.0);
}