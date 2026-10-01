use bytemuck::NoUninit;
use cgmath::{Vector3, prelude::*};
use spheres::physics::*;
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor,
    CurrentSurfaceTexture::*, Device, DeviceDescriptor, FragmentState, Instance,
    InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureViewDescriptor, VertexBufferLayout, VertexState,
    VertexStepMode, vertex_attr_array,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const MAX_BALLS: usize = 10_000;
const METERS_PER_PIXEL: f64 = 0.001;

#[repr(C, packed)]
#[derive(NoUninit, Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    texture_position: [f32; 2],
}

enum SphereStep {
    None,
    Size,
}

enum Game<'s> {
    Uninitialized,
    Initialized {
        pause: bool,
        energies: Vec<f64>,
        sphere_step: SphereStep,
        queue: Queue,
        device: Device,
        window: Arc<Window>,
        buffer: Buffer,
        surface: Surface<'s>,
        instance: Instance,
        shader: ShaderModule,
        instant: Instant,
        spheres: Vec<Sphere>,
        pipeline: RenderPipeline,
        mouse_world_position: Vector3<f64>,
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
            size: size_of::<Vertex>() as u64 * 6 * MAX_BALLS as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let [width, height] = window.inner_size().into();

        let mut surface_configuration =
            surface.get_default_config(&adapter, width, height).unwrap();
        // surface_configuration.present_mode = PresentMode::AutoVsync;

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
                    array_stride: size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
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
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview_mask: None,
            cache: None,
        });

        *self = Game::Initialized {
            pause: false,
            sphere_step: SphereStep::None,
            energies: vec![],
            queue,
            device,
            buffer,
            window,
            surface,
            instance,
            shader,
            pipeline,
            instant: Instant::now(),
            spheres: vec![],
            mouse_world_position: Vector3::new(0.0, 0.0, 0.0),
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
            WindowEvent::CursorMoved { position, .. } => {
                if let Game::Initialized {
                    mouse_world_position,
                    surface_configuration,
                    ..
                } = self
                {
                    mouse_world_position.x = position.x as f64 * METERS_PER_PIXEL;
                    mouse_world_position.y = (surface_configuration.height as f64
                        - position.y as f64)
                        * METERS_PER_PIXEL;
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Game::Initialized { pause, .. } = self {
                    *pause = !*pause;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                if let Game::Initialized {
                    mouse_world_position,
                    spheres,
                    sphere_step,
                    ..
                } = self
                {
                    match sphere_step {
                        SphereStep::None => {
                            let new_spheres: Vec<_> = spheres
                                .iter()
                                .filter(|sphere| {
                                    (sphere.position - *mouse_world_position).magnitude()
                                        > sphere.radius
                                })
                                .map(|&sphere| sphere.clone())
                                .collect();
                            if new_spheres.len() < spheres.len() {
                                *spheres = new_spheres;
                            } else {
                                spheres.push(Sphere {
                                    position: *mouse_world_position,
                                    velocity: Vector3::zero(),
                                    radius: 50.0 / METERS_PER_PIXEL, // TODO: Change radius by
                                                                     // scroll, or live update a
                                                                     // see through ball with
                                                                     // radius
                                });
                                *sphere_step = SphereStep::Size;
                            }
                        }
                        SphereStep::Size => {
                            let r = (spheres.last().unwrap().position - *mouse_world_position)
                                .magnitude();
                            spheres.last_mut().unwrap().radius = r;
                            *sphere_step = SphereStep::None;
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Game::Initialized {
                    sphere_step,
                    surface,
                    queue,
                    device,
                    window,
                    mouse_world_position,
                    pipeline,
                    buffer,
                    spheres,
                    instant,
                    pause,
                    energies,
                    surface_configuration,
                    ..
                } = self
                {
                    match surface.get_current_texture() {
                        Success(texture) => {
                            // Pixels
                            let screen_width = surface_configuration.width as f32;
                            let screen_height = surface_configuration.height as f32;

                            // Meters
                            let box_width = screen_width as f64 * METERS_PER_PIXEL;
                            let box_height = screen_height as f64 * METERS_PER_PIXEL;

                            let delta_time = instant.elapsed();
                            *instant = Instant::now();

                            let borders = vec![
                                Border {
                                    position: 0.0,
                                    normal: Vector3::new(0.0, 1.0, 0.0),
                                },
                                Border {
                                    position: 0.0,
                                    normal: Vector3::new(1.0, 0.0, 0.0),
                                },
                                Border {
                                    position: box_height,
                                    normal: Vector3::new(0.0, -1.0, 0.0),
                                },
                                Border {
                                    position: box_width,
                                    normal: Vector3::new(-1.0, 0.0, 0.0),
                                },
                                // TODO: borders on z-axis
                            ];

                            match (*pause, &sphere_step) {
                                (false, SphereStep::None) if spheres.len() >= 2 => {
                                    let b = tick(spheres, &borders, delta_time, energies);

                                    // if b {
                                    //     *pause = true;
                                    // }
                                }
                                _ => (),
                            }

                            let mut spheres = spheres.clone();
                            match &sphere_step {
                                SphereStep::None => spheres.push(Sphere {
                                    position: *mouse_world_position,
                                    radius: 0.0,
                                    velocity: Vector3::zero(),
                                }),
                                SphereStep::Size => {
                                    let sphere = spheres.last_mut().unwrap();
                                    sphere.radius = sphere.position.distance(*mouse_world_position);
                                }
                            }
                            let verticies: Vec<Vertex> = spheres
                                .iter()
                                .map(
                                    |Sphere {
                                         position: Vector3 { x, y, .. },
                                         radius,
                                         ..
                                     }| {
                                        let x_min = x - radius;
                                        let x_max = x + radius;
                                        let y_min = y - radius;
                                        let y_max = y + radius;

                                        let x_min = x_min / METERS_PER_PIXEL;
                                        let x_max = x_max / METERS_PER_PIXEL;
                                        let y_min = y_min / METERS_PER_PIXEL;
                                        let y_max = y_max / METERS_PER_PIXEL;

                                        let x_min = (x_min as f32 / screen_width - 0.5) * 2.0;
                                        let x_max = (x_max as f32 / screen_width - 0.5) * 2.0;
                                        let y_min = (y_min as f32 / screen_height - 0.5) * 2.0;
                                        let y_max = (y_max as f32 / screen_height - 0.5) * 2.0;

                                        (x_min, x_max, y_min, y_max)
                                    },
                                )
                                .map(|(x_min, x_max, y_min, y_max)| {
                                    [
                                        Vertex {
                                            position: [x_min, y_min],
                                            texture_position: [-1.0, -1.0],
                                        },
                                        Vertex {
                                            position: [x_min, y_max],
                                            texture_position: [-1.0, 1.0],
                                        },
                                        Vertex {
                                            position: [x_max, y_max],
                                            texture_position: [1.0, 1.0],
                                        },
                                        Vertex {
                                            position: [x_max, y_max],
                                            texture_position: [1.0, 1.0],
                                        },
                                        Vertex {
                                            position: [x_max, y_min],
                                            texture_position: [1.0, -1.0],
                                        },
                                        Vertex {
                                            position: [x_min, y_min],
                                            texture_position: [-1.0, -1.0],
                                        },
                                    ]
                                })
                                .flatten()
                                .collect();
                            let mut command_encoder =
                                device.create_command_encoder(&CommandEncoderDescriptor::default());
                            {
                                let view = &texture.texture.create_view(&TextureViewDescriptor {
                                    format: Some(surface_configuration.format),
                                    ..Default::default()
                                });

                                let mut render_pass =
                                    command_encoder.begin_render_pass(&RenderPassDescriptor {
                                        color_attachments: &vec![Some(RenderPassColorAttachment {
                                            view: &view,
                                            depth_slice: None,
                                            resolve_target: None,
                                            ops: Operations {
                                                load: LoadOp::Clear(Color::BLACK),
                                                store: StoreOp::Store,
                                            },
                                        })],
                                        ..Default::default()
                                    });

                                let _ = render_pass.set_vertex_buffer(0, buffer.slice(..));

                                render_pass.set_pipeline(&pipeline);
                                render_pass.draw(0..(spheres.len() as u32 * 6), 0..1);
                            }

                            queue.write_buffer(&buffer, 0, &bytemuck::cast_slice(&verticies));
                            let command_buffers = vec![command_encoder.finish()];
                            queue.submit(command_buffers);

                            // window.pre_present_notify();
                            queue.present(texture);

                            window.request_redraw();
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
                // dbg!(event);
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::Uninitialized;
    let _ = event_loop.run_app(&mut game);
}
