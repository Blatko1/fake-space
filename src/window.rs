use winit::{dpi::PhysicalSize, window::Window as InnerWindow};

pub struct Window {
    surface: wgpu::Surface,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    winit_window: InnerWindow,
}

impl Window {
    pub async fn init(
        winit_window: InnerWindow,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backends = wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::PRIMARY);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = unsafe { instance.create_surface(&winit_window) }?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Request Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let size = winit_window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            config,
            queue,
            winit_window,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn recreate_sc(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn inner_window(&self) -> &InnerWindow {
        &self.winit_window
    }
}
