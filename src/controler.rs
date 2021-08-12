use wgpu::{util::DeviceExt, vertex_attr_array};

use crate::settings::{WINDOW_HEIGHT, WINDOW_WIDHT};

pub struct Controler {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    base_render_buffer: wgpu::Buffer,
    base_render_pipeline: wgpu::RenderPipeline,
}

impl Controler {
    pub async fn new(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: WINDOW_WIDHT,
            height: WINDOW_HEIGHT,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let base_render_shader_module =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("base-render-shader.wgsl").into()),
                flags: wgpu::ShaderFlags::all(),
            });

        let base_render_buffer_data = [0.0f32, 0.5, -0.5, -0.5, 0.5, -0.5];
        let base_render_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Base-Render-Buffer"),
            contents: bytemuck::cast_slice(&base_render_buffer_data),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE,
        });
        let base_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let base_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Base-Render-Pipeline"),
            layout: Some(&base_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &base_render_shader_module,
                entry_point: "base_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 2 * 4,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0=>Float32x2],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
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
                module: &base_render_shader_module,
                entry_point: "base_main",
                targets: &[sc_desc.format.into()],
            }),
        });

        Controler {
            device,
            queue,
            swap_chain,
            base_render_buffer,
            base_render_pipeline,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command-Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render-Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.base_render_pipeline);
            render_pass.set_vertex_buffer(0, self.base_render_buffer.slice(..));

            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
