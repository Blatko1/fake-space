use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode},
};
use crate::window::{Vertex, Window};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

pub struct State {
    window: Window,
    pipeline: wgpu::RenderPipeline,
    canvas_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,

    should_exit: bool,
}

impl State {
    pub fn new(window: Window) -> Self {

        let sampler = window.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Canvas Texture Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let size = wgpu::Extent3d {
            width: window.config().width,
            height: window.config().height,
            depth_or_array_layers: 1,
        };

        let texture = window.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let size = (size.width * size.height) as usize;
        let mut content: Vec<f32> = Vec::with_capacity(WIDTH * HEIGHT * 4);
        for i in 0..size {
            content.push(0.5);
            content.push(0.6);
            content.push(0.7);
            content.push(1.0);
        }

        window.queue().write_texture(wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        }, bytemuck::cast_slice(&content), wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: todo!(),
            rows_per_image: todo!(),
        }, size)

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader = window
            .device()
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let bind_group_layout = window.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                ],
            },
        );

        let bind_group = window.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view)
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler)
            }],
        });

        let pipeline_layout = window.device().create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        );

        let pipeline = window.device().create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::buffer_layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: window.config().format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        );

        let canvas_triangle = [
            Vertex {
                pos: [-1.0, -1.0, 0.0], // bottom-left
                tex_pos: [0.0, 1.0],
            },
            Vertex {
                pos: [3.0, -1.0, 0.0], // bottom-right
                tex_pos: [4.0, 1.0],
            },
            Vertex {
                pos: [-1.0, 3.0, 0.0], // top-left
                tex_pos: [0.0, 4.0],
            },
        ];

        let canvas_buffer = window.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Canvas Buffer"),
                contents: bytemuck::cast_slice(&canvas_triangle),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        Self {
            window,
            pipeline,
            canvas_buffer,

            bind_group,
            texture,
            
            should_exit: false,
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.window.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            },
        );

        let frame = self.window.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
            rpass.set_vertex_buffer(0, self.canvas_buffer.slice(..));
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.draw(0..3, 0..1);
        }

        self.window.queue().submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size)
    }

    pub fn process_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
            self.should_exit = true;
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}
