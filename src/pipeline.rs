use super::Size2D;
use crate::canvas::Canvas;

// Output of raytracer before postprocessing
pub(crate) struct RawOutput {
    texture: wgpu::Texture,
    bindgroup: wgpu::BindGroup,
}

impl RawOutput {
    const LABEL: &str = "upscale-input";
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    fn bind_group_layout(canvas: &Canvas) -> wgpu::BindGroupLayout {
        canvas
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(Self::LABEL),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: Self::FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            })
    }

    fn new(canvas: &Canvas, size: Size2D) -> Self {
        let texture = canvas.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(Self::LABEL),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(Self::LABEL),
            format: Some(Self::FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let bindgroup = canvas.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("upscale-input"),
            layout: &Self::bind_group_layout(canvas),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        Self { texture, bindgroup }
    }
    pub(crate) fn resize(&mut self, canvas: &Canvas, size: Size2D) {
        *self = Self::new(canvas, size);
    }
}

struct Raytracer {
    pub(crate) pipeline: wgpu::ComputePipeline,
}

impl Raytracer {
    const LABEL: &'static str = "raytracer";

    fn new(canvas: &Canvas, module: &wgpu::ShaderModule) -> Self {
        let layout = canvas
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(Self::LABEL),
                bind_group_layouts: &[&RawOutput::bind_group_layout(canvas)],
                push_constant_ranges: &[],
            });

        let pipeline = canvas
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(Self::LABEL),
                layout: Some(&layout),
                module,
                entry_point: "raytrace",
            });

        Self { pipeline }
    }
}

struct PostProcess {
    pipeline: wgpu::RenderPipeline,
}

impl PostProcess {
    const LABEL: &str = "postprocess";
    
    fn new(canvas: &Canvas, module: &wgpu::ShaderModule) -> Self {
        let layout = canvas
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(Self::LABEL),
                bind_group_layouts: &[&RawOutput::bind_group_layout(canvas)],
                push_constant_ranges: &[],
            });

        let pipeline = canvas
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(Self::LABEL),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: canvas.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self { pipeline }
    }
}

pub(crate) struct Pipeline {
    raytracer: Raytracer,
    postprocess: PostProcess,
    rawoutput: RawOutput,

    size: Size2D,
    scale: u32,
}

impl Pipeline {
    fn downscale(size: Size2D, scale: u32) -> Size2D {
        Size2D {
            width: size.width / scale,
            height: size.width / scale,
        }
    }

    pub(crate) fn new(canvas: &Canvas) -> Self {
        let scale = 8;
        let size = Self::downscale(canvas.get_size(), scale);

        let module = canvas.load_shader("shaders/raytracer.wgsl");

        let raytracer = Raytracer::new(canvas, &module);
        let postprocess = PostProcess::new(canvas, &module);
        let intermediate = RawOutput::new(canvas, size);

        Self {
            raytracer,
            postprocess,
            rawoutput: intermediate,

            size,
            scale,
        }
    }

    pub(crate) fn render(&self, canvas: &Canvas, view: &wgpu::TextureView) {
        let mut encoder = canvas
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pipeline.encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("raytracer"),
            });

            let workgroup = (16, 16);
            let dim = Self::downscale(self.size, self.scale * 16);

            pass.set_bind_group(0, &self.rawoutput.bindgroup, &[]);
            pass.set_pipeline(&self.raytracer.pipeline);
            pass.dispatch_workgroups(dim.width / workgroup.0, dim.height / workgroup.1, 1);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("postprocess"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_bind_group(0, &self.rawoutput.bindgroup, &[]);
            pass.set_pipeline(&self.postprocess.pipeline);
        }

        canvas.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn resize(&mut self, canvas: &Canvas, size: Size2D) {
        self.size = size;
        self.rawoutput.resize(&canvas, Self::downscale(size, self.scale));        
    }
}
