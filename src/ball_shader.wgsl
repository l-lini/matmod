@binding(0) @group(0) var<uniform> camera_matrix: mat4x4f;

struct Vertex {
    @location(0) position: vec4f,
    @location(1) texture_position: vec2f,
}

struct Instance {
    @location(2) matrix_1: vec4f,
    @location(3) matrix_2: vec4f,
    @location(4) matrix_3: vec4f,
    @location(5) matrix_4: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> VertexOutput {
    var vertex_output: VertexOutput;

    let matrix = mat4x4<f32>(
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
        instance.matrix_4,
    );

    vertex_output.position = camera_matrix * matrix * vertex.position;
    vertex_output.texture_position = vertex.texture_position;

    return vertex_output;
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texture_position: vec2f,
};

fn inverse(m: mat4x4f) -> mat4x4f {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

    return mat4x4f(
        a11 * b11 - a12 * b10 + a13 * b09,
        a02 * b10 - a01 * b11 - a03 * b09,
        a31 * b05 - a32 * b04 + a33 * b03,
        a22 * b04 - a21 * b05 - a23 * b03,
        a12 * b08 - a10 * b11 - a13 * b07,
        a00 * b11 - a02 * b08 + a03 * b07,
        a32 * b02 - a30 * b05 - a33 * b01,
        a20 * b05 - a22 * b02 + a23 * b01,
        a10 * b10 - a11 * b08 + a13 * b06,
        a01 * b08 - a00 * b10 - a03 * b06,
        a30 * b04 - a31 * b02 + a33 * b00,
        a21 * b02 - a20 * b04 - a23 * b00,
        a11 * b07 - a10 * b09 - a12 * b06,
        a00 * b09 - a01 * b07 + a02 * b06,
        a31 * b01 - a30 * b03 - a32 * b00,
        a20 * b03 - a21 * b01 + a22 * b00) * (1 / det);
  }

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var a: f32;
    var r: f32 = length(vertex.texture_position);
    if length(vertex.texture_position) > 1.0 {
		discard;
    } else if length(vertex.texture_position) > 0.99 {
        a = 1.0 - smoothstep(0.99, 1.0, r);
    } else {
        a = 1.0;
    }
    var z: f32 = sqrt(1.0 - clamp(r * r, 0.0, 1.0));
    var n = 0.5 + 0.5 * vec3f(vertex.texture_position, z);
    var light4 = inverse(camera_matrix) * vec4f(- 200.0, 200.0, 200.0, 1.0);
    var light3 = normalize(vec3f(light4.x, light4.y, light4.z));
    var brightness: f32 = clamp(dot(light3, n), 0.1, 1.0);
    var color = vec3(0.1, 0.843, 1.0);
    return vec4f(color * brightness * a, a);
}
