struct VertexOutput {
    [[location(0)]] color: vec3<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};
// struct VertexOutput {
//     [[location(0)]] tex_coor: vec2<f32>;
//     [[builtin(position)]] pos: vec4<f32>;
// };

[[block]]
struct CellUniformData {
    data: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> cell_data: CellUniformData;

[[stage(vertex)]]
fn base_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] input_coor: vec2<f32>,
    [[location(2)]] input_color: vec3<f32>,
    // [[location(0)]] pos: vec2<f32>,
    // [[location(1)]] tex_coor: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = input_color;
    // out.tex_coor = tex_coor;
    out.pos = vec4<f32>(pos.x + input_coor.x * cell_data.data.x - 1.0, pos.y - input_coor.y * cell_data.data.y + 1.0, 0.0, 1.0);
    // out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

// [[group(0), binding(0)]] var tex_color: texture_2d<f32>;
// [[group(0), binding(1)]] var tex_sampler: sampler;

[[stage(fragment)]]
fn base_main([[location(0)]] color: vec3<f32>) -> [[location(0)]] vec4<f32> {
// fn base_main([[location(0)]] tex_coor: vec2<f32>) -> [[location(0)]] vec4<f32> {
    var res: vec3<f32> = clamp(sqrt(color), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0));
    return vec4<f32>(res, 1.0);
    // return textureSample(tex_color, tex_sampler, tex_coor);
}
