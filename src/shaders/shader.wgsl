struct Global {
    matrix: mat4x4<f32>
}
@group(0) @binding(2)
var<uniform> global: Global;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_pos: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = global.matrix * vec4<f32>(in.pos, 0.0, 1.0);
    //out.tex_pos = fma(inpos, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    //if (pos.x < 0.0) {
    //    if (pos.y < 0.0) {
    //        out.tex_pos.x = out.tex_pos.x - 0.5;
    //    }
    //    
    //}    
    //if (pos.x > 0.0) {
    //    if (pos.y < 0.0) {
    //        out.tex_pos.x = out.tex_pos.x + 0.5;
    //    }
    //    
    //}
    out.tex_pos = in.tex_pos;

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