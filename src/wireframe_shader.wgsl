@binding(0) @group(0) var<uniform> camera_matrix: mat4x4f;

@vertex
fn vs_main(@location(0) position: vec4f) -> @builtin(position) vec4<f32> {
    return camera_matrix * position;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4f(1.0, 1.0, 1.0, 1.0);
}
