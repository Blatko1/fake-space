struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_pos: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.pos, 1.0);
    out.tex_pos = in.tex_pos;

    return out;   
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(0.3, 0.2, 0.1, 1.0);
    let rgb = textureSample(texture, t_sampler, in.tex_pos).rgb;
    return vec4<f32>(rgb, 1.0);
}