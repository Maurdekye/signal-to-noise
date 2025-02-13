struct Uniforms {
    resolution: vec2<f32>,
    cell_spacing: f32,
    signal_origin: vec2<f32>,
    signal_strength: f32,
    signal_width: f32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
    dimensions: u32,
}

@group(3) @binding(0)
var<uniform> params: Uniforms;

fn smooth_clamp(x: f32, a: f32) -> f32 {
    let z = clamp(x / a, -1.5, 1.5);
    return a * (z - ((4.0 / 27.0) * z * z * z));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / params.resolution;
    let cell_id: vec2<f32> = select(
        vec2<f32>(floor(uv.x / params.cell_spacing), 0.0),
        floor(uv / params.cell_spacing),
        params.dimensions == 2u
    );

    let base_seed = dot(cell_id, vec2<f32>(12.9898, 78.233));
    let seed = base_seed + params.noise_seed;

    let u1 = fract(sin(seed) * 43758.5453);
    let u2 = fract(sin(seed + 1.0) * 43758.5453);

    let z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * 3.14159265 * u2);
    let z0_clamped = smooth_clamp(z0, params.noise_deviation_cap);
    let noise_strength = params.noise_deviation * z0_clamped + params.noise_floor;

    let clamped_position = (cell_id + 0.5) * params.cell_spacing;
    let signal_origin_dist: f32 = select(
        length(clamped_position.x - params.signal_origin.x),
        length(clamped_position - params.signal_origin),
        params.dimensions == 2u
    );
    let factor = signal_origin_dist / params.signal_width;
    let signal_strength = exp(-factor * factor) * params.signal_strength;

    let strength = noise_strength + signal_strength;

    if params.dimensions == 1u {
        let shade = select(1.0, 0.0, strength < 1.0 - uv.y);
        return vec4<f32>(shade, shade, shade, 1.0);
    } else {
        return vec4<f32>(strength, strength, strength, 1.0);
    }
}