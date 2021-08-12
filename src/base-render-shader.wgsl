struct VertexInput {
    [[location(0)]] pos: vec2<f32>;
};

[[stage(vertex)]]
fn base_main(vertex_in: VertexInput) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(vertex_in.pos, 0.0, 1.0);
}

[[stage(fragment)]]
fn base_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.3, 0.7, 1.0);
}
