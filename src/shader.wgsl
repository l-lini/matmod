struct Vertex {
    @location(0) position: vec2f,
    @location(1) texture_position: vec2f,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;
    vertex_output.position = vec4f(vertex.position, 0.0, 1.0);
    vertex_output.texture_position = vertex.texture_position;

    return vertex_output;
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texture_position: vec2f,
};

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var a: f32;
    var r: f32 = length(vertex.texture_position);
    if length(vertex.texture_position) > 1.0 {
		discard;
    } else if length(vertex.texture_position) > 0.95 {
        a = 1.0 - smoothstep(0.95, 1.0, r);
    } else {
        a = 1.0;
    }
    var z: f32 = sqrt(1.0 - clamp(r * r, 0.0, 1.0));
    var n = 0.5 + 0.5 * vec3f(vertex.texture_position, z);
    var light = vec3(- 200.0, 200.0, 200.0);
    light = normalize(light);
    var brightness: f32 = clamp(dot(light, n), 0.1, 1.0);
    var color = vec3(0.1, 0.843, 1.0);
    return vec4f(color * brightness * a, a);
}
