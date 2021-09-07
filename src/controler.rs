use std::time::Instant;

use wgpu::{util::DeviceExt, vertex_attr_array};

use crate::{
    entity::{Panel, Sphere},
    settings::{
        IMAGE_PATH, SAMPLES_PER_PIXEL, TEXTURE_HEIGHT, TEXTURE_WIDTH, WINDOW_HEIGHT,
        WINDOW_TOTAL_PIXEL, WINDOW_WIDHT,
    },
    systems::generator::{
        generate_input_data, generate_lights_scene, generate_panel_scene, generate_sphere_scene,
    },
};

pub struct Controler {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    cell_render_bind_group: wgpu::BindGroup,
    cell_render_buffer: wgpu::Buffer,
    cell_render_pipeline: wgpu::RenderPipeline,
    entity_buffers: Vec<wgpu::Buffer>,
    input_buffer: wgpu::Buffer,
    result_buffer: wgpu::Buffer,
    // source_texture: wgpu::Texture,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bindgroup0: wgpu::BindGroup,
    compute_bindgroup1: wgpu::BindGroup,
    work_group_count: u32,
    // clip_rect: (u32, u32, u32, u32),
}

impl Controler {
    pub async fn new(window: &winit::window::Window) -> Self {
        // let clip_rect = (0u32, 0u32, WINDOW_WIDHT, WINDOW_HEIGHT);
        let instance = wgpu::Instance::new(wgpu::Backends::METAL);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
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
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: WINDOW_WIDHT,
            height: WINDOW_HEIGHT,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let compute_shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute-Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute-shader.wgsl").into()),
        });

        let base_render_shader_module =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Render-Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("base-render-shader.wgsl").into()),
            });
        // let source_texture = device.create_texture(&wgpu::TextureDescriptor {
        //     label: None,
        //     size: wgpu::Extent3d {
        //         width: TEXTURE_WIDTH,
        //         height: TEXTURE_HEIGHT,
        //         // width: WINDOW_WIDHT,
        //         // height: WINDOW_HEIGHT,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        // });
        // let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     label: None,
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     lod_min_clamp: 0.0,
        //     lod_max_clamp: 1.0,
        //     compare: None,
        //     anisotropy_clamp: None,
        //     border_color: None,
        // });
        let cell_width = 2.0 / (TEXTURE_WIDTH as f32);
        let cell_height = 2.0 / (TEXTURE_HEIGHT as f32);
        let cell_buffer_data: [[f32; 2]; 6] = [
            [0.0, 0.0],
            [0.0, -cell_height],
            [cell_width, -cell_height],
            [0.0, 0.0],
            [cell_width, -cell_height],
            [cell_width, 0.0],
        ];
        // let cell_buffer_data: [[[f32; 2]; 2]; 3] = [
        //     // One full-screen triangle
        //     // See: https://github.com/parasyte/pixels/issues/180
        //     [[-1.0, -1.0], [0.0, 0.0]],
        //     [[3.0, -1.0], [2.0, 0.0]],
        //     [[-1.0, 3.0], [0.0, 2.0]],
        // ];
        let cell_uniform_buffer_data = [cell_width, cell_height];
        let cell_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cell_uniform_buffer_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
        });
        let cell_render_bindgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(2 * std::mem::size_of::<f32>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                }],
            });
        let cell_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &cell_render_bindgroup_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cell_uniform_buffer.as_entire_binding(),
            }],
        });
        // let cell_render_bindgroup_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: None,
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStage::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture {
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                     multisampled: false,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStage::FRAGMENT,
        //                 ty: wgpu::BindingType::Sampler {
        //                     filtering: true,
        //                     comparison: false,
        //                 },
        //                 count: None,
        //             },
        //         ],
        //     });
        // let cell_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: None,
        //     layout: &cell_render_bindgroup_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(
        //                 &source_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        //             ),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&texture_sampler),
        //         },
        //     ],
        // });
        let cell_render_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Base-Render-Buffer"),
            contents: bytemuck::cast_slice(&cell_buffer_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cell_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&cell_render_bindgroup_layout],
                push_constant_ranges: &[],
            });
        let cell_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cell-Render-Pipeline"),
            layout: Some(&cell_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &base_render_shader_module,
                entry_point: "base_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0=>Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &vertex_attr_array![1=>Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 3 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &vertex_attr_array![2=>Float32x3],
                    },
                ],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &base_render_shader_module,
                entry_point: "base_main",
                targets: &[surface.get_preferred_format(&adapter).unwrap().into()],
            }),
        });
        // let cell_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Cell-Render-Pipeline"),
        //     layout: Some(&cell_render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &base_render_shader_module,
        //         entry_point: "base_main",
        //         buffers: &[wgpu::VertexBufferLayout {
        //             array_stride: 4 * 4,
        //             step_mode: wgpu::InputStepMode::Vertex,
        //             attributes: &vertex_attr_array![0=>Float32x2, 1=>Float32x2],
        //         }],
        //     },
        //     primitive: wgpu::PrimitiveState::default(),
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState::default(),
        //     fragment: Some(wgpu::FragmentState {
        //         module: &base_render_shader_module,
        //         entry_point: "base_main",
        //         targets: &[sc_desc.format.into()],
        //     }),
        // });

        let sphere_buffer_data = generate_sphere_scene();
        let mut entity_buffers = vec![];
        entity_buffers.push(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&sphere_buffer_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            }),
        );
        let panel_buffer_data = generate_panel_scene();
        entity_buffers.push(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&panel_buffer_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            }),
        );
        let light_buffer_data = generate_lights_scene();
        entity_buffers.push(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&light_buffer_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            }),
        );
        let mut config_buffer_data = vec![];
        config_buffer_data.push(0u32);
        config_buffer_data.push(WINDOW_WIDHT);
        config_buffer_data.push(WINDOW_HEIGHT);
        config_buffer_data.push(SAMPLES_PER_PIXEL as u32);
        config_buffer_data.push(sphere_buffer_data.len() as u32);
        config_buffer_data.push(panel_buffer_data.len() as u32);
        config_buffer_data.push(light_buffer_data.len() as u32);
        config_buffer_data.push(0u32);
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&config_buffer_data),
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
        });

        let input_buffer_data = generate_input_data();
        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input-Buffer"),
            contents: bytemuck::cast_slice(&input_buffer_data),
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX,
        });

        let result_buffer_data = vec![0.0f32; (WINDOW_TOTAL_PIXEL * 3) as usize];
        // let result_buffer_data = vec![0u8; (WINDOW_TOTAL_PIXEL * 4) as usize];
        let result_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Res-Buffer"),
            contents: bytemuck::cast_slice(&result_buffer_data),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX,
        });

        let compute_bindgroup0_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(WINDOW_TOTAL_PIXEL * 2 * 4).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(WINDOW_TOTAL_PIXEL * 3 * 4).unwrap(),
                                // std::num::NonZeroU64::new(WINDOW_TOTAL_PIXEL * 4 * 1).unwrap(),
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let compute_bindgroup0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute-Bindgroup"),
            layout: &compute_bindgroup0_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });
        let compute_bindgroup1_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(
                                    (sphere_buffer_data.len() * std::mem::size_of::<Sphere>())
                                        as u64,
                                )
                                .unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(
                                    (panel_buffer_data.len() * std::mem::size_of::<Panel>()) as u64,
                                )
                                .unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(
                                    (light_buffer_data.len() * std::mem::size_of::<Panel>()) as u64,
                                )
                                .unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(
                                    (config_buffer_data.len() * std::mem::size_of::<u32>()) as u64,
                                )
                                .unwrap(),
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let compute_bindgroup1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entity"),
            layout: &compute_bindgroup1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: entity_buffers[0].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: entity_buffers[1].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: entity_buffers[2].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: config_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&compute_bindgroup0_layout, &compute_bindgroup1_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader_module,
            entry_point: "main",
        });

        let work_group_count = ((WINDOW_TOTAL_PIXEL as f32) / 64.0).ceil() as u32;

        Controler {
            device,
            queue,
            surface,
            cell_render_bind_group,
            cell_render_buffer,
            cell_render_pipeline,
            entity_buffers,
            input_buffer,
            result_buffer,
            // source_texture,
            compute_pipeline,
            compute_bindgroup0,
            compute_bindgroup1,
            work_group_count,
            // clip_rect,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let surface_frame = self.surface.get_current_frame().expect("Error get current frame");
        let view = surface_frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute-Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute-Pass"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bindgroup0, &[]);
            compute_pass.set_bind_group(1, &self.compute_bindgroup1, &[]);
            compute_pass.dispatch(self.work_group_count, 1, 1);
        }
        let compute_end = Instant::now();

        // encoder.copy_buffer_to_texture(
        //     wgpu::ImageCopyBuffer {
        //         buffer: &self.result_buffer,
        //         layout: wgpu::ImageDataLayout {
        //             offset: 0,
        //             bytes_per_row: std::num::NonZeroU32::new(4 * TEXTURE_WIDTH),
        //             rows_per_image: std::num::NonZeroU32::new(TEXTURE_HEIGHT),
        //         },
        //     },
        //     wgpu::ImageCopyTexture {
        //         texture: &self.source_texture,
        //         mip_level: 1,
        //         origin: wgpu::Origin3d::ZERO,
        //     },
        //     wgpu::Extent3d {
        //         width: TEXTURE_WIDTH / 2,
        //         height: TEXTURE_HEIGHT / 2,
        //         depth_or_array_layers: 1,
        //     },
        // );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render-Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.cell_render_pipeline);
            render_pass.set_bind_group(0, &self.cell_render_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.cell_render_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.input_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.result_buffer.slice(..));
            render_pass.draw(0..6, 0..WINDOW_TOTAL_PIXEL as u32);
            // render_pass.set_scissor_rect(
            //     self.clip_rect.0,
            //     self.clip_rect.1,
            //     self.clip_rect.2,
            //     self.clip_rect.3,
            // );
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
