use super::Label;

use super::Size2D;
use std::path::Path;
use winit::window::Window;

// A combined surface and rendering context which
/// manages all the wgpu handles requires for rendering
pub(crate) struct Canvas {
    pub(crate) label: Label,
    pub(crate) instance: wgpu::Instance,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) surface: wgpu::Surface,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Canvas {
    pub(crate) async fn new(label: &Label, window: &Window) -> Self {
        let backends = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backends);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // Prefer discrete GPUs
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Context: failed to find appropriate GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(label.sublabel("device").as_str()),
                    features:
                        // Required for read-write storage buffer 
                        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Context: failed to find appropriate GPU device");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        Self {
            label: label.to_owned(),
            instance,
            device,
            queue,
            surface,
            config,
        }
    }

    pub(crate) fn get_size(&self) -> Size2D {
        Size2D::new(self.config.width, self.config.height)
    }

    pub(crate) fn resize(&mut self, size: Size2D) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config)
        }
    }

    pub(crate) fn load_shader<P: AsRef<Path>>(&self, path: P) -> wgpu::ShaderModule {
      let source = std::fs::read_to_string(path.as_ref())
          .expect(format!("Canvas: unable to load shader at [{:?}]", path.as_ref()).as_str());

      let label = path.as_ref().to_string_lossy();

      self.device
          .create_shader_module(wgpu::ShaderModuleDescriptor {
              label: Some(self.label.sublabel(label).as_str()),
              source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(source)),
          })
    }
}
