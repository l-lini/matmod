use cgmath::{Matrix4, Rad, Vector2, Vector3, perspective, prelude::*};
use spheres::graphics::*;
use spheres::physics::*;
use std::{f32::consts::PI, time::Instant};
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
            balls: vec![Sphere {
                position: Vector3::zero(),
                velocity: Vector3::zero(),
                radius: 0.5,
            }],
            walls: vec![
                Border {
                    position: 1.0,
                    normal: Vector3::new(0.0, 1.0, 0.0),
                },
                Border {
                    position: 1.0,
                    normal: Vector3::new(1.0, 0.0, 0.0),
                },
                Border {
                    position: 1.0,
                    normal: Vector3::new(0.0, -1.0, 0.0),
                },
                Border {
                    position: 1.0,
                    normal: Vector3::new(-1.0, 0.0, 0.0),
                },
                Border {
                    position: 1.0,
                    normal: Vector3::new(0.0, 0.0, 1.0),
                },
                Border {
                    position: 1.0,
                    normal: Vector3::new(0.0, 0.0, -1.0),
                },
            ],
            gravity: 9.82,
            energies: vec![],
        };

        let camera_position = (3.0, 1.5, 3.0).into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = cgmath::Vector3::unit_y();
        let view_matrix = Matrix4::look_at_rh(camera_position, look_direction, up_direction);
        let box_verticies = [
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
        ]
        .map(|[x, y, z]| BoxVertex {
            position: [x, y, z, 1.0],
        });
        let graphics = Graphics::new(
            event_loop,
            view_matrix,
            Rad(2.0 * PI / 5.0),
            0.1,
            100.0,
            &box_verticies,
        );

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
                let verticies: Vec<BallVertex> = physics
                    .balls
                    .iter()
                    .map(
                        |Sphere {
                             position: Vector3 { x, y, z },
                             radius,
                             ..
                         }| {
                            let bottom_left = [x - radius, y - radius, *z, -1.0, -1.0];
                            let bottom_right = [x + radius, y - radius, *z, 1.0, -1.0];
                            let top_left = [x - radius, y + radius, *z, -1.0, 1.0];
                            let top_right = [x + radius, y + radius, *z, 1.0, 1.0];

                            [
                                bottom_left,
                                top_left,
                                top_right,
                                bottom_left,
                                top_right,
                                bottom_right,
                            ]
                        },
                    )
                    .flatten()
                    .map(|[x, y, z, uvx, uvy]| BallVertex {
                        position: [x as f32, y as f32, z as f32, 1.0],
                        texture_position: [uvx as f32, uvy as f32],
                    })
                    .collect();
                graphics.set_ball_verticies(&verticies);
                graphics.draw();
            }
            (e, _) => {
                dbg!(e);
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::Uninitialized;
    let _ = event_loop.run_app(&mut game);
}
