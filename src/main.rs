use bytemuck::{NoUninit, Pod, bytes_of};
use std::{borrow::Cow, sync::Arc};
use tokio::runtime::Runtime;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor,
    CurrentSurfaceTexture::*, Device, DeviceDescriptor, FragmentState, Instance,
    InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureViewDescriptor, VertexBufferLayout, VertexState, VertexStepMode,
    vertex_attr_array,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[repr(C, packed)]
#[derive(NoUninit, Copy, Clone)]
struct Vertex {
    x: f32,
    y: f32,
}

enum Game<'s> {
    Uninitialized,
    Initialized {
        queue: Queue,
        device: Device,
        window: Arc<Window>,
        buffer: Buffer,
        surface: Surface<'s>,
        instance: Instance,
        shader: ShaderModule,
        pipeline: RenderPipeline,
        surface_configuration: SurfaceConfiguration,
    },
}

impl<'s> ApplicationHandler for Game<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let display = event_loop.owned_display_handle();

        let instance = Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
            display,
        )));

        let surface = instance.create_surface(window.clone()).unwrap();

        let rt = Runtime::new().unwrap();

        let adapter = rt
            .block_on(instance.request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .unwrap();

        let (device, queue) = rt
            .block_on(adapter.request_device(&DeviceDescriptor::default()))
            .unwrap();

        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 100,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let [width, height] = window.inner_size().into();

        let surface_configuration = surface.get_default_config(&adapter, width, height).unwrap();

        surface.configure(&device, &surface_configuration);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &vec![],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vec![Some(VertexBufferLayout {
                    array_stride: 8 as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32,
                        1 => Float32
                    ],
                })],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &vec![Some(TextureFormat::Bgra8UnormSrgb.into())],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview_mask: None,
            cache: None,
        });

        *self = Game::Initialized {
            queue,
            device,
            buffer,
            window,
            surface,
            instance,
            shader,
            pipeline,
            surface_configuration,
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Game::Initialized {
                    surface,
                    device,
                    surface_configuration,
                    ..
                } = self
                {
                    surface_configuration.width = size.width;
                    surface_configuration.height = size.height;
                    surface.configure(&device, &surface_configuration);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Game::Initialized {
                    surface,
                    queue,
                    device,
                    window,
                    pipeline,
                    buffer,
                    ..
                } = self
                {
                    match surface.get_current_texture() {
                        Success(texture) => {
                            let mut command_encoder =
                                device.create_command_encoder(&CommandEncoderDescriptor::default());
                            {
                                let view = &texture.texture.create_view(&TextureViewDescriptor {
                                    format: Some(TextureFormat::Bgra8UnormSrgb),
                                    ..Default::default()
                                });

                                let mut render_pass =
                                    command_encoder.begin_render_pass(&RenderPassDescriptor {
                                        color_attachments: &vec![Some(RenderPassColorAttachment {
                                            view: &view,
                                            depth_slice: None,
                                            resolve_target: None,
                                            ops: Operations {
                                                load: LoadOp::Clear(Color::WHITE),
                                                store: StoreOp::Store,
                                            },
                                        })],
                                        ..Default::default()
                                    });

                                let _ = render_pass.set_vertex_buffer(0, buffer.slice(..));

                                render_pass.set_pipeline(&pipeline);
                                render_pass.draw(0..3, 0..1);
                            }

                            let verticies = [
                                Vertex { x: 0.0, y: 0.0 },
                                Vertex { x: 0.0, y: -0.5 },
                                Vertex { x: -0.5, y: -0.5 },
                                Vertex { x: 0.0, y: 0.0 },
                                Vertex { x: 1.0, y: 1.0 },
                                Vertex { x: 0.0, y: 1.0 },
                            ];
                            let bytes: Vec<u8> = verticies
                                .iter()
                                .map(|vertex| [vertex.x, vertex.y])
                                .flatten()
                                .map(|float| float.to_bits().to_ne_bytes())
                                .flatten()
                                .collect();
                            let floats = queue.write_buffer(&buffer, 0, &bytes);
                            let command_buffers = vec![command_encoder.finish()];
                            queue.submit(command_buffers);

                            // window.pre_present_notify();
                            queue.present(texture);

                            window.request_redraw()
                        }
                        Suboptimal(_) => todo!("highly recomended to reconfigure"),
                        Timeout => todo!("skip and try again later"),
                        Occluded => todo!("skip frame and wait until not occluded"),
                        Outdated => todo!("configure and try again"),
                        Lost => todo!("check if device lost, try to recreate surface"),
                        Validation => todo!("attend to the validation error"),
                    }
                }
            }
            _ => {
                dbg!(event);
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::Uninitialized;
    let _ = event_loop.run_app(&mut game);
}
