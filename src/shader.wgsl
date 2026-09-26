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
	if length(vertex.texture_position) > 1.0 {
		discard;
	} else {
		return vec4f(vertex.texture_position, 1.0, 1.0);
	}
}
