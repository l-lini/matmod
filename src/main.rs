use cgmath::{Vector2, Vector3, prelude::*};
use spheres::graphics::*;
use spheres::physics::*;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowId,
};

const METERS_PER_PIXEL: f64 = 0.01;

enum SphereStep {
    None,
    Size,
}

enum Game<'surface> {
    Uninitialized,
    Initialized {
        pause: bool,
        physics: Physics,
        graphics: Graphics<'surface>,
        sphere_step: SphereStep,
        instant: Instant,
        mouse_position: Vector2<f64>,
    },
}

impl<'s> ApplicationHandler for Game<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let physics = Physics {
            balls: vec![],
            walls: vec![
                Border {
                    position: 0.0,
                    normal: Vector3::new(0.0, 1.0, 0.0),
                },
                Border {
                    position: 0.0,
                    normal: Vector3::new(1.0, 0.0, 0.0),
                },
                Border {
                    position: 3.0,
                    normal: Vector3::new(0.0, -1.0, 0.0),
                },
                Border {
                    position: 3.0,
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                // TODO: borders on z-axis
            ],
            gravity: 9.82,
            energies: vec![],
        };

        let graphics = Graphics::new(event_loop);

        *self = Self::Initialized {
            graphics,
            physics,
            pause: false,
            sphere_step: SphereStep::None,
            instant: Instant::now(),
            mouse_position: Vector2::zero(),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match (event, self) {
            (WindowEvent::CloseRequested, _) => event_loop.exit(),
            (WindowEvent::Resized(size), Game::Initialized { graphics, .. }) => {
                graphics.resize(size.width, size.height);
            }
            (
                WindowEvent::CursorMoved { position, .. },
                Game::Initialized { mouse_position, .. },
            ) => {
                mouse_position.x = position.x;
                mouse_position.y = position.y;
            }
            (
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Space),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                },
                Game::Initialized { pause, .. },
            ) => {
                *pause = !*pause;
            }
            (
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                },
                Game::Initialized {
                    mouse_position,
                    physics,
                    sphere_step,
                    ..
                },
            ) => {
                let mouse_world_position =
                    Vector3::new(mouse_position.x, mouse_position.y, 0.0) * METERS_PER_PIXEL;

                match sphere_step {
                    SphereStep::None => {
                        let new_spheres: Vec<_> = physics
                            .balls
                            .iter()
                            .filter(|sphere| {
                                (sphere.position - mouse_world_position).magnitude() > sphere.radius
                            })
                            .map(|&sphere| sphere.clone())
                            .collect();
                        if new_spheres.len() < physics.balls.len() {
                            physics.balls = new_spheres;
                        } else {
                            physics.balls.push(Sphere {
                                position: mouse_world_position,
                                velocity: Vector3::zero(),
                                radius: 0.0,
                            });
                            *sphere_step = SphereStep::Size;
                        }
                    }
                    SphereStep::Size => {
                        let r = (physics.balls.last().unwrap().position - mouse_world_position)
                            .magnitude();
                        physics.balls.last_mut().unwrap().radius = r;
                        *sphere_step = SphereStep::None;
                        dbg!(physics.balls.last().unwrap());
                    }
                }
            }
            (
                WindowEvent::RedrawRequested,
                Game::Initialized {
                    graphics,
                    sphere_step,
                    mouse_position,
                    physics,
                    instant,
                    pause,
                    ..
                },
            ) => {
                let mouse_world_position =
                    Vector3::new(mouse_position.x, mouse_position.y, 0.0) * METERS_PER_PIXEL;

                let delta_time = instant.elapsed();
                *instant = Instant::now();

                match (*pause, &sphere_step) {
                    (false, SphereStep::None) => _ = physics.tick(delta_time),
                    _ => (),
                }
                let mut spheres = physics.balls.clone();
                match &sphere_step {
                    SphereStep::None => spheres.push(Sphere {
                        position: mouse_world_position,
                        radius: 0.0,
                        velocity: Vector3::zero(),
                    }),
                    SphereStep::Size => {
                        let sphere = spheres.last_mut().unwrap();
                        sphere.radius = sphere.position.distance(mouse_world_position);
                    }
                }
                graphics.verticies = spheres
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

                            let screen_width = 500.0;
                            let screen_height = 1080.0;

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
                                position: [x_min, y_min, 0.0],
                                texture_position: [-1.0, -1.0],
                            },
                            Vertex {
                                position: [x_min, y_max, 0.0],
                                texture_position: [-1.0, 1.0],
                            },
                            Vertex {
                                position: [x_max, y_max, 0.0],
                                texture_position: [1.0, 1.0],
                            },
                            Vertex {
                                position: [x_max, y_max, 0.0],
                                texture_position: [1.0, 1.0],
                            },
                            Vertex {
                                position: [x_max, y_min, 0.0],
                                texture_position: [1.0, -1.0],
                            },
                            Vertex {
                                position: [x_min, y_min, 0.0],
                                texture_position: [-1.0, -1.0],
                            },
                        ]
                    })
                    .flatten()
                    .collect();
                graphics.draw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::Uninitialized;
    let _ = event_loop.run_app(&mut game);
}
