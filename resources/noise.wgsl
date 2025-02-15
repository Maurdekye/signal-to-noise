const GUASSIAN: u32 = 0u;
const PARETO: u32 = 1u;
const TRIANGLE: u32 = 2u;
const UNIFORM: u32 = 3u;

struct Uniforms {
    resolution: vec2<f32>,
    cell_spacing: f32,
    signal_origin: vec2<f32>,
    signal_strength: f32,
    signal_width: f32,
    signal_shape: u32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
    noise_distribution: u32,
    noise_pareto_distribution_parameter: f32,
    dimensions: u32,
}

@group(3) @binding(0)
var<uniform> params: Uniforms;

fn rand(seed: f32) -> f32 {
    return fract(sin(seed) * 43758.5453);
}

fn random_gaussian(seed: f32) -> f32 {
    let u1 = rand(seed);
    let u2 = rand(seed + 1.0);
    return sqrt(-2.0 * log(u1)) * cos(2.0 * 3.14159265 * u2);
}

fn random_pareto(seed: f32, alpha: f32) -> f32 {
    let u = rand(seed);
    return 1.0 / pow((1.0 - u), (1.0 / (alpha * alpha))) - 1.0;
}

fn random_triangle(seed: f32) -> f32 {
    let u1 = rand(seed);
    let u2 = rand(seed + 1.0);
    return u1 + u2 - 1.0;
}

fn random_uniform(seed: f32) -> f32 {
    return rand(seed) - 0.5;
}

fn gaussian_distribution(x: f32) -> f32 {
    const P: f32 = 0.564189583548;
    let x0 = x * P;
    return exp(-x0 * x0);
}

fn pareto_distribution(x: f32) -> f32 {
    const A: f32 = 100.0;
    return pow(A / (A + abs(x)), 2.0 * A + 1.0);
}

fn triangle_distribution(x: f32) -> f32 {
    return max(0.0, 1.0 - abs(x));
}

fn uniform_distribution(x: f32) -> f32 {
    return select(0.0, 1.0, abs(x) <= 0.5);
}

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

    var random_var = 0.0;
    if params.noise_distribution == GUASSIAN {
        random_var = random_gaussian(seed);
    } else if params.noise_distribution == PARETO {
        random_var = random_pareto(seed, params.noise_pareto_distribution_parameter);
    } else if params.noise_distribution == TRIANGLE {
        random_var = random_triangle(seed);
    } else if params.noise_distribution == UNIFORM {
        random_var = random_uniform(seed);
    }

    let clamped_random_var = smooth_clamp(random_var, params.noise_deviation_cap);

    let noise_strength = params.noise_deviation * clamped_random_var + params.noise_floor;

    let clamped_position = (cell_id + 0.5) * params.cell_spacing;
    let signal_origin_dist: f32 = select(
        length(clamped_position.x - params.signal_origin.x),
        length(clamped_position - params.signal_origin),
        params.dimensions == 2u
    );
    let factor = signal_origin_dist / params.signal_width;

    var signal_distribution = 0.0;
    if params.signal_shape == GUASSIAN {
        signal_distribution = gaussian_distribution(factor);
    } else if params.signal_shape == PARETO {
        signal_distribution = pareto_distribution(factor);
    } else if params.signal_shape == TRIANGLE {
        signal_distribution = triangle_distribution(factor);
    } else if params.signal_shape == UNIFORM {
        signal_distribution = uniform_distribution(factor);
    }

    let signal_strength = signal_distribution * params.signal_strength;

    let strength = noise_strength + signal_strength;

    if params.dimensions == 1u {
        let shade = select(1.0, 0.0, strength < 1.0 - uv.y);
        return vec4<f32>(shade, shade, shade, 1.0);
    } else {
        return vec4<f32>(strength, strength, strength, 1.0);
    }
}