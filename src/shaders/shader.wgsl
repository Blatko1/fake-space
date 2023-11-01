struct Global {
    matrix: mat4x4<f32>
}
@group(0) @binding(2)
var<uniform> global: Global;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = global.matrix * vec4<f32>(pos, 0.0, 1.0);
    out.tex_pos = fma(pos, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));

    return out;   
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(texture, t_sampler, in.tex_pos);
    return texel;
}