struct Uniforms {
    x: f32
}

@group(3) @binding(0)
var<uniform> params: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(params.x, params.x, params.x, 1.0);
}