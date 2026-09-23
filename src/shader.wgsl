struct Vertex {
	@location(0) x: f32,
	@location(1) y: f32,
};

@vertex
fn vs_main(vertex: Vertex) -> @builtin(position) vec4f {
	return vec4f(vertex.x, vertex.y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}
