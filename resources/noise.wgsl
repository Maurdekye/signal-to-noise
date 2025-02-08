struct Uniform {
    resolution: vec2<f32>,
    grid_spacing: f32,
    shift_duration: f32,
    time: f32,
}

@group(3) @binding(0)
var<uniform> params: Uniform;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / params.resolution;
    let grid_coord = floor(uv / params.grid_spacing);
    let time_frame = floor(params.time / params.shift_duration);
    
    let base_seed = dot(grid_coord, vec2<f32>(12.9898, 78.233));
    let seed = base_seed + time_frame;

    let shade = fract(sin(seed) * 43758.5453);
    
    return vec4<f32>(shade, shade, shade, 1.0);
}