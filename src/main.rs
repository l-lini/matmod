use bytemuck::NoUninit;
use cgmath::{Vector2, prelude::*};
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

#[repr(C, packed)]
#[derive(NoUninit, Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    texture_position: [f32; 2],
}

enum SphereStep {
    None,
    Size,
    Speed,
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
        mouse_position: Vector2<f64>,
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
            mouse_position: Vector2::new(0.0, 0.0),
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
                if let Game::Initialized { mouse_position, .. } = self {
                    mouse_position.x = position.x as f64;
                    mouse_position.y = position.y as f64;
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
                    surface_configuration,
                    mouse_position,
                    spheres,
                    sphere_step,
                    ..
                } = self
                {
                    let mut mouse_world_position = Vector2::new(
                        mouse_position.x as f64,
                        surface_configuration.height as f64 - mouse_position.y,
                    );
                    match sphere_step {
                        SphereStep::None => {
                            let new_spheres: Vec<_> = spheres
                                .iter()
                                .filter(|sphere| {
                                    (sphere.position - mouse_world_position).magnitude()
                                        > sphere.radius
                                })
                                .map(|&sphere| sphere.clone())
                                .collect();
                            if new_spheres.len() < spheres.len() {
                                *spheres = new_spheres;
                            } else {
                                spheres.push(Sphere {
                                    position: mouse_world_position,
                                    velocity: Vector2::new(0.0, 0.0),
                                    radius: 100.0,
                                });
                                *sphere_step = SphereStep::Size;
                            }
                        }
                        SphereStep::Size => {
                            let r = (spheres.last().unwrap().position - mouse_world_position)
                                .magnitude();
                            spheres.last_mut().unwrap().radius = r;
                            *sphere_step = SphereStep::Speed;
                        }
                        SphereStep::Speed => {
                            let mut velocity =
                                mouse_world_position - spheres.last().unwrap().position;
                            spheres.last_mut().unwrap().velocity = velocity;

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
                            let width = surface_configuration.width as f32;
                            let height = surface_configuration.height as f32;

                            let delta_duration = instant.elapsed();
                            *instant = Instant::now();

                            let borders = vec![
                                Border {
                                    position: Vector2::new(0.0, 0.0),
                                    normal: Vector2::new(0.0, 1.0),
                                },
                                Border {
                                    position: Vector2::new(0.0, 0.0),
                                    normal: Vector2::new(1.0, 0.0),
                                },
                                Border {
                                    position: Vector2::new(0.0, height as f64),
                                    normal: Vector2::new(0.0, -1.0),
                                },
                                Border {
                                    position: Vector2::new(width as f64, 0.0),
                                    normal: Vector2::new(-1.0, 0.0),
                                },
                            ];

                            match (*pause, sphere_step) {
                                (false, SphereStep::None) if spheres.len() >= 2 => {
                                    let b = tick(spheres, &borders, delta_duration, energies);

                                    // if b {
                                    //     *pause = true;
                                    // }
                                }
                                _ => (),
                            }

                            let verticies: Vec<Vertex> = spheres
                                .iter()
                                .map(
                                    |Sphere {
                                         position: Vector2 { x, y },
                                         radius,
                                         ..
                                     }| {
                                        (x - radius, y - radius, x + radius, y + radius)
                                    },
                                )
                                .map(|(x_min, y_min, x_max, y_max)| {
                                    (
                                        x_min as i32 - width as i32 / 2,
                                        y_min as i32 - height as i32 / 2,
                                        x_max as i32 - width as i32 / 2,
                                        y_max as i32 - height as i32 / 2,
                                    )
                                })
                                .map(
                                    |(
                                        centered_x_min,
                                        centered_y_min,
                                        centered_x_max,
                                        centered_y_max,
                                    )| {
                                        (
                                            centered_x_min as f32 / width,
                                            centered_y_min as f32 / height,
                                            centered_x_max as f32 / width,
                                            centered_y_max as f32 / height,
                                        )
                                    },
                                )
                                .map(|(x, y, x_, y_)| (2.0 * x, 2.0 * y, 2.0 * x_, 2.0 * y_))
                                .map(|(x_min, y_min, x_max, y_max)| {
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
