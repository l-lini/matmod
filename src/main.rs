use cgmath::{Matrix4, Point3, Rad, Vector2, Vector3, perspective, prelude::*};
use spheres::graphics::*;
use spheres::physics::*;
use std::{f32::consts::PI, time::Duration, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowId,
};

enum Game<'surface> {
    Uninitialized,
    Initialized {
        pause: bool,
        physics: Physics,
        graphics: Graphics<'surface>,
        camera: Vector2<Rad<f32>>,
        instant: Instant,
        right: bool,
        mouse_position: Vector2<f64>,
    },
}

impl<'s> ApplicationHandler for Game<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let physics = Physics {
            balls: vec![
                Sphere {
                    position: Vector3::zero(),
                    velocity: Vector3::zero(),
                    radius: 0.5,
                },
                Sphere {
                    position: Vector3::new(0.5, 0.5, 0.75),
                    velocity: Vector3::new(0.1, 0.0, 0.0),
                    radius: 0.4,
                },
            ],
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

        let camera = Vector2::new(Rad(0.0), Rad(PI / 4.0));
        let view_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, -4.0))
            * Matrix4::from_angle_x(camera.x)
            * Matrix4::from_angle_y(camera.y);
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
        let ball_verticies: Vec<_> = {
            let bottom_left = [-1.0, -1.0, 0.0, -1.0, -1.0];
            let bottom_right = [1.0, -1.0, 0.0, 1.0, -1.0];
            let top_left = [-1.0, 1.0, 0.0, -1.0, 1.0];
            let top_right = [1.0, 1.0, 0.0, 1.0, 1.0];

            [
                bottom_left,
                top_left,
                top_right,
                bottom_left,
                top_right,
                bottom_right,
            ]
        }
        .iter()
        .map(|[x, y, z, uvx, uvy]| BallVertex {
            position: [*x, *y, *z, 1.0],
            texture_position: [*uvx, *uvy],
        })
        .collect();
        let graphics = Graphics::new(
            event_loop,
            view_matrix,
            Rad(2.0 * PI / 5.0),
            0.1,
            100.0,
            &box_verticies,
            &ball_verticies,
        );

        *self = Self::Initialized {
            graphics,
            physics,
            pause: false,
            instant: Instant::now(),
            right: false,
            camera,
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
                Game::Initialized {
                    mouse_position,
                    right,
                    camera,
                    graphics,
                    ..
                },
            ) => {
                if *right {
                    camera.x -= Rad((mouse_position.y as f32 - position.y as f32) * 0.01);
                    camera.y -= Rad((mouse_position.x as f32 - position.x as f32) * 0.01);
                }
                let view_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, -4.0))
                    * Matrix4::from_angle_x(camera.x)
                    * Matrix4::from_angle_y(camera.y);
                graphics.set_view_matrix(&view_matrix);
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
                    button: MouseButton::Right,
                    ..
                },
                Game::Initialized { right, .. },
            ) => {
                *right = true;
            }
            (
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Right,
                    ..
                },
                Game::Initialized { right, .. },
            ) => {
                *right = false;
            }
            (
                WindowEvent::RedrawRequested,
                Game::Initialized {
                    graphics,
                    mouse_position,
                    physics,
                    instant,
                    camera,
                    pause,
                    ..
                },
            ) => {
                physics.tick(Duration::from_secs_f32(
                    instant.elapsed().as_secs_f32() / 2.0,
                ));
                *instant = Instant::now();
                let instances: Vec<[[f32; 4]; 4]> = physics
                    .balls
                    .iter()
                    .map(|ball| {
                        let position = Vector3::new(
                            ball.position.x as f32,
                            ball.position.y as f32,
                            ball.position.z as f32,
                        );
                        let translation = Matrix4::from_translation(position);
                        let scale = Matrix4::from_scale(ball.radius as f32);
                        let rotation =
                            Matrix4::from_angle_y(-camera.y) * Matrix4::from_angle_x(-camera.x);

                        let matrix = translation * rotation * scale;

                        matrix.into()
                    })
                    .collect();
                graphics.set_instances(&instances);
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
