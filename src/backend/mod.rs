pub mod ctx;
mod debug;

use debug::DebugUI;
use pollster::block_on;
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};
use std::ptr;
use wgpu::util::DeviceExt;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use crate::{CANVAS_HEIGHT_FACTOR, CANVAS_WIDTH_FACTOR};

use self::ctx::Ctx;

const TRIANGLE_VERTICES: [[f32; 2]; 3] = [
    [-1.0, -1.0], // bottom-left
    [3.0, -1.0],  // bottom-right
    [-1.0, 3.0],  // top-left
];

// TODO better explanation
pub struct Canvas {
    buffer: Vec<u8>,
    frame: Vec<u8>,
    view_width: u32,
    view_height: u32,

    ctx: Ctx,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    region: ScissorRegion,
    vertex_buffer: wgpu::Buffer,
    matrix_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    texture: wgpu::Texture,
    size: wgpu::Extent3d,

    debug_ui: DebugUI,
}

impl Canvas {
    const CANVAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(
        event_loop: &ActiveEventLoop,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        let ctx = block_on(Ctx::new(event_loop)).unwrap();
        let device = ctx.device();
        let render_format = ctx.config().format;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Canvas Texture Sampler"),
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            ..Default::default()
        });

        let size = wgpu::Extent3d {
            width: canvas_width,
            height: canvas_height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::CANVAS_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let matrix_buffer_size =
            std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress;
        let matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Matrix Uniform Buffer"),
            size: matrix_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Main Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: std::num::NonZeroU64::new(
                                matrix_buffer_size,
                            ),
                        },
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: matrix_buffer.as_entire_binding(),
                },
            ],
        });

        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&TRIANGLE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let shader: wgpu::ShaderModule =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        let buffer_size = (canvas_width * canvas_height * 3) as usize;
        let frame_size = (canvas_width * canvas_height * 4) as usize;

        // TODO change/fix this
        let font_data = std::fs::read("tiled/Minecraft.ttf").unwrap();
        let font = FontVec::try_from_vec(font_data).unwrap();
        let debug_ui = DebugUI::new(&ctx, font);

        Self {
            // RGB - 3 bytes per pixel
            buffer: vec![255; buffer_size],
            // RGBA - 4 bytes per pixel
            frame: vec![255; frame_size],
            view_width: canvas_width,
            view_height: canvas_height,

            ctx,
            pipeline,
            bind_group_layout,
            bind_group,

            region: ScissorRegion::default(),
            vertex_buffer,
            matrix_buffer,
            sampler,
            texture,
            size,

            debug_ui,
        }
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.fill(0);

        // TODO cool effects!
        //self.buffer.try_fill(&mut rand::thread_rng()).unwrap();
    }

    pub fn mut_column_iterator(&mut self) -> impl Iterator<Item = &mut [u8]> {
        self.buffer.chunks_exact_mut(self.view_height as usize * 3)
    }

    pub fn mut_column(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Flip the buffer texture to correct position (90 degrees anticlockwise)
        self.frame
            .chunks_exact_mut(self.view_width as usize * 4)
            .rev()
            .enumerate()
            .for_each(|(x, row)| {
                self.buffer
                    .chunks_exact(3)
                    .skip(x)
                    .step_by(self.view_height as usize)
                    .zip(row.chunks_exact_mut(4))
                    .for_each(|(src, dest)| {
                        //dest[0..3].copy_from_slice(src);
                        unsafe {
                            ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), 3);
                        }
                    })
            });

        self.ctx.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&self.frame),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.view_width * 4),
                rows_per_image: None,
            },
            self.size,
        );

        let mut encoder =
            self.ctx
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        let frame = self.ctx.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_scissor_rect(
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            );
            rpass.draw(0..3, 0..1);
            self.debug_ui.render(&mut rpass);
        }

        self.ctx.queue().submit(Some(encoder.finish()));

        self.ctx.window().pre_present_notify();

        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        let config = self.ctx.config();

        let window_width = config.width as f32;
        let window_height = config.height as f32;
        let (texture_width, texture_height) =
            (self.view_width as f32, self.view_height as f32);

        let scale = (window_width / texture_width)
            .min(window_height / texture_height)
            .max(1.0);
        let scaled_width = texture_width * scale;
        let scaled_height = texture_height * scale;

        let s_w = scaled_width / window_width;
        let s_h = scaled_height / window_height;
        let t_x = (window_width / 2.0).fract() / window_width;
        let t_y = (window_height / 2.0).fract() / window_height;
        let matrix = [
            [s_w, 0.0, 0.0, 0.0],
            [0.0, s_h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [t_x, t_y, 0.0, 1.0],
        ];

        self.ctx.queue().write_buffer(
            &self.matrix_buffer,
            0,
            bytemuck::cast_slice(&matrix),
        );

        self.region = ScissorRegion {
            x: ((window_width - scaled_width) / 2.0).floor() as u32,
            y: ((window_height - scaled_height) / 2.0).floor() as u32,
            width: scaled_width.min(window_width) as u32,
            height: scaled_height.min(window_height) as u32,
        };

        self.debug_ui.resize(self.region, &self.ctx);
    }

    pub fn toggle_full_screen(&mut self) {
        self.ctx.toggle_full_screen();
    }

    pub fn increase_resolution(&mut self) {
        let device = self.ctx.device();
        self.view_width += CANVAS_WIDTH_FACTOR;
        self.view_height += CANVAS_HEIGHT_FACTOR;
        let (size, texture, view) =
            Self::create_frame_texture(device, self.view_width, self.view_height);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.matrix_buffer.as_entire_binding(),
                },
            ],
        });
        self.size = size;
        self.texture = texture;
        self.bind_group = bind_group;

        let buffer_len = (self.view_width * self.view_height * 3) as usize;
        let frame_len = (self.view_width * self.view_height * 4) as usize;
        self.buffer = vec![255; buffer_len];
        self.frame = vec![255; frame_len];
    }

    pub fn decrease_resolution(&mut self) {
        let device = self.ctx.device();
        self.view_width = self
            .view_width
            .saturating_sub(CANVAS_WIDTH_FACTOR)
            .max(CANVAS_WIDTH_FACTOR);
        self.view_height = self
            .view_height
            .saturating_sub(CANVAS_HEIGHT_FACTOR)
            .max(CANVAS_HEIGHT_FACTOR);
        let (size, texture, view) =
            Self::create_frame_texture(device, self.view_width, self.view_height);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.matrix_buffer.as_entire_binding(),
                },
            ],
        });
        self.size = size;
        self.texture = texture;
        self.bind_group = bind_group;

        let buffer_len = (self.view_width * self.view_height * 3) as usize;
        let frame_len = (self.view_width * self.view_height * 4) as usize;
        self.buffer = vec![255; buffer_len];
        self.frame = vec![255; frame_len];
    }

    pub fn create_frame_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Extent3d, wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::CANVAS_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (size, texture, view)
    }

    pub fn request_redraw(&self) {
        self.ctx.window().request_redraw()
    }

    pub fn on_surface_lost(&self) {
        self.ctx.recreate_sc()
    }

    pub fn view_width(&self) -> u32 {
        self.view_width
    }

    pub fn view_height(&self) -> u32 {
        self.view_height
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ScissorRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
