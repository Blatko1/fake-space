use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode},
};

use crate::window::Window;

pub struct State {
    window: Window,

    should_exit: bool
}

impl State {
    pub fn new(window: Window) -> Self {
        Self { window, should_exit: false }
    }

    pub fn render(&self) -> Result<(),wgpu::SurfaceError> {
        let mut encoder = self.window.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        let frame = self.window.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
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
