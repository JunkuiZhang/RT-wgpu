struct VertexOutput {
    [[location(0)]] tex_coor: vec2<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[block]]
struct CellUniformData {
    data: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> cell_data: CellUniformData;

[[stage(vertex)]]
fn base_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] tex_coor: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coor.x = tex_coor.x;
    out.tex_coor.y = 1.0 - tex_coor.y;
    out.pos = vec4<f32>(pos.x, pos.y, 0.0, 1.0);
    return out;
}

[[group(0), binding(0)]] var tex_color: texture_2d<f32>;
[[group(0), binding(1)]] var tex_sampler: sampler;

[[stage(fragment)]]
fn base_main([[location(0)]] tex_coor: vec2<f32>) -> [[location(0)]] vec4<f32> {
    return textureSample(tex_color, tex_sampler, tex_coor);
}
