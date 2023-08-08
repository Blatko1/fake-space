use wgpu::util::DeviceExt;

use crate::window::Window;

const TRIANGLE_VERTICES: [[f32; 2]; 3] = [
    [-1.0, -1.0], // bottom-left
    [3.0, -1.0],  // bottom-right
    [-1.0, 3.0],  // top-left
];

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Default)]
struct ScissorRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Canvas {
    data: Vec<Pixel>,
    width: u32,
    height: u32,

    pipeline: wgpu::RenderPipeline,
    canvas_bind_group: wgpu::BindGroup,
    fill_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,

    region: ScissorRegion,
    vertex_buffer: wgpu::Buffer,
    canvas_matrix_buffer: wgpu::Buffer,
    fill_matrix_buffer: wgpu::Buffer,

    canvas_texture: DrawTexture,
    fill_texture: DrawTexture,
}

impl Canvas {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        render_format: wgpu::TextureFormat,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        let canvas_texture =
            DrawTexture::new_canvas_tex(device, canvas_width, canvas_height);
        let fill_texture = DrawTexture::new_fill_tex(
            device,
            config.width,
            config.height,
            render_format,
        );

        let matrix_buffer_size =
            std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress;
        let canvas_matrix_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Canvas Matrix Uniform Buffer"),
                size: matrix_buffer_size,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        let fill_matrix_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Fill Matrix Uniform Buffer"),
                size: matrix_buffer_size,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&TRIANGLE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let shader: wgpu::ShaderModule = device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

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

        let canvas_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Canvas Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &canvas_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &canvas_texture.sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: canvas_matrix_buffer.as_entire_binding(),
                    },
                ],
            });

        let fill_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fill Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &fill_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &fill_texture.sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: fill_matrix_buffer.as_entire_binding(),
                    },
                ],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>()
                            as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    }],
                },
                primitive: wgpu::PrimitiveState::default(),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: render_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                depth_stencil: None,
                multiview: None,
            });

        Self {
            data: vec![
                Pixel::default();
                (canvas_width * canvas_height) as usize
            ],
            width: canvas_width,
            height: canvas_height,

            pipeline,
            canvas_bind_group,
            fill_bind_group,
            bind_group_layout,

            region: ScissorRegion::default(),
            vertex_buffer,
            canvas_matrix_buffer,
            fill_matrix_buffer,

            canvas_texture,
            fill_texture,
        }
    }

    pub fn clear_data(&mut self) {
        self.data.fill(Pixel::default());
    }

    pub fn data_mut(&mut self) -> &mut [Pixel] {
        &mut self.data
    }

    pub fn render(&self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        window.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.canvas_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&self.data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: None,
            },
            self.canvas_texture.size,
        );

        let mut encoder = window.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            },
        );

        let frame = window.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main RenderPass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &self.fill_texture.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.canvas_bind_group, &[]);
            rpass.set_scissor_rect(
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            );
            rpass.draw(0..3, 0..1);
        }

        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main RenderPass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.fill_bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        window.queue().submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn resize(
        // TODO maybe take window dimensions as two arguments
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let window_width = config.width as f32;
        let window_height = config.height as f32;
        let (texture_width, texture_height) =
            (self.width as f32, self.height as f32);

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

        queue.write_buffer(
            &self.canvas_matrix_buffer,
            0,
            bytemuck::cast_slice(&matrix),
        );

        self.region = ScissorRegion {
            x: ((window_width - scaled_width) / 2.0).floor() as u32,
            y: ((window_height - scaled_height) / 2.0).floor() as u32,
            width: scaled_width.min(window_width) as u32,
            height: scaled_height.min(window_height) as u32,
        };
        self.resize_fill(device, queue, config);
    }

    pub fn resize_fill(
        // TODO maybe take window dimensions as two arguments
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration
    ) {
        // TODO change this possibly (arguments; it would be better to provide only width and height)
        self.fill_texture = DrawTexture::new_fill_tex(device, config.width, config.height, config.format);
        self.fill_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fill Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.fill_texture.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &self.fill_texture.sampler,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.fill_matrix_buffer.as_entire_binding(),
                },
            ],
        });
        
        let window_width = config.width as f32;
        let window_height = config.height as f32;
        let (texture_width, texture_height) =
            (self.region.width as f32, self.region.height as f32);

        let scale = (window_width / texture_width)
            .min(window_height / texture_height);
        let scaled_width = texture_width * scale;
        let scaled_height = texture_height * scale;

        let s_w = scaled_width / window_width;
        let s_h = scaled_height / window_height;
        let t_x = (window_width / 2.0).fract() / window_width;
        let t_y = (window_height / 2.0).fract() / window_height;
        let matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [t_x, t_y, 0.0, 1.0],
        ];

        // TODO this is temp!!
        queue.write_buffer(
            &self.fill_matrix_buffer,
            0,
            bytemuck::cast_slice(&matrix),
        );
    }
}

struct DrawTexture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: wgpu::Extent3d,
}

impl DrawTexture {
    const CANVAS_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Rgba8UnormSrgb;

    fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sampler_desc: wgpu::SamplerDescriptor,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let sampler = device.create_sampler(&sampler_desc);
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
            format,
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            sampler,
            texture,
            view,
            size,
        }
    }

    #[inline]
    fn new_canvas_tex(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let sampler_desc = wgpu::SamplerDescriptor {
            label: Some("Canvas Texture Sampler"),
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            ..Default::default()
        };
        let usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST;
        Self::new(
            device,
            width,
            height,
            Self::CANVAS_FORMAT,
            sampler_desc,
            usage,
        )
    }

    #[inline]
    fn new_fill_tex(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let sampler_desc = wgpu::SamplerDescriptor {
            label: Some("Fill Texture Sampler"),
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        };
        let usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT;
        Self::new(device, width, height, format, sampler_desc, usage)
    }
}
