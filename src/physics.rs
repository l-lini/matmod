use cgmath::{Vector2, prelude::*};
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub radius: f32,
    pub mass: f32,
}

impl Sphere {
    fn time_travel(&mut self, seconds: f32) {
        self.velocity += self.acceleration * seconds / 2.0;
        self.position += self.velocity * seconds;
        self.velocity += self.acceleration * seconds / 2.0;
    }
}

pub struct Border {
    pub normal: Vector2<f32>,
    pub position: Vector2<f32>,
}

pub struct Collision {
    seconds_until: f32,
    sphere_index: usize,
    object: CollisionObject,
}

pub enum CollisionObject {
    Sphere(usize),
    Border(usize),
}

pub fn seconds_until_border(sphere: &Sphere, border: &Border) -> Option<f32> {
    let v = sphere.velocity.dot(border.normal);
    let p = (sphere.position - border.position).dot(border.normal);
    let t = -(p - sphere.radius) / v;

    if v < 0.0 { Some(t) } else { None }
}

pub fn seconds_until_sphere(sphere: &Sphere, other_sphere: &Sphere) -> Option<f32> {
    let r = sphere.radius + other_sphere.radius;
    let v = other_sphere.velocity - sphere.velocity;
    let p = other_sphere.position - sphere.position;

    let a = v.x.powi(2) + v.y.powi(2);
    let b = 2.0 * p.x * v.x + 2.0 * p.y * v.y;
    let c = p.x.powi(2) + p.y.powi(2) - r.powi(2);

    let slope = |t| 2.0 * a * t + b;

    let radicand = b.powi(2) - 4.0 * a * c;

    if radicand < 0.0 {
        return None;
    }

    let t = (-b - radicand.sqrt()) / (2.0 * a);

    if slope(t) < 0.0 {
        return Some(t);
    }

    let t = (-b + radicand.sqrt()) / (2.0 * a);

    if slope(t) < 0.0 {
        return Some(t);
    }

    return None;
}

pub fn next_collision(
    spheres: &[Sphere],
    borders: &[Border],
    min_seconds: f32,
    max_seconds: f32,
) -> Option<Collision> {
    let mut collision = None;

    for (sphere_index, sphere) in spheres.iter().enumerate() {
        for (border_index, border) in borders.iter().enumerate() {
            let t = seconds_until_border(sphere, border);

            match (t, &collision) {
                (Some(t), None) if t > min_seconds && t < max_seconds => {
                    collision = Some(Collision {
                        seconds_until: t,
                        sphere_index,
                        object: CollisionObject::Border(border_index),
                    })
                }
                (Some(t), Some(Collision { seconds_until, .. }))
                    if t > min_seconds && t < max_seconds && t < *seconds_until =>
                {
                    collision = Some(Collision {
                        seconds_until: t,
                        sphere_index,
                        object: CollisionObject::Border(border_index),
                    })
                }
                _ => (),
            }
        }

        for (other_sphere_index, other_sphere) in spheres.iter().enumerate().skip(sphere_index + 1)
        {
            let t = seconds_until_sphere(&sphere, &other_sphere);

            match (t, &collision) {
                (Some(t), None) if t > min_seconds && t < max_seconds => {
                    collision = Some(Collision {
                        seconds_until: t,
                        sphere_index,
                        object: CollisionObject::Sphere(other_sphere_index),
                    })
                }
                (Some(t), Some(Collision { seconds_until, .. }))
                    if t > min_seconds && t < max_seconds && t < *seconds_until =>
                {
                    collision = Some(Collision {
                        seconds_until: t,
                        sphere_index,
                        object: CollisionObject::Sphere(other_sphere_index),
                    })
                }
                _ => (),
            }
        }
    }

    collision
}

pub fn tick(spheres: &mut Vec<Sphere>, borders: &[Border], delta_duration: Duration) {
    let mut delta_seconds = delta_duration.as_secs_f32();

    while let Some(Collision {
        seconds_until,
        sphere_index,
        object,
    }) = next_collision(&spheres, &borders, -delta_seconds, delta_seconds)
    {
        delta_seconds -= seconds_until;

        for sphere_index in 0..spheres.len() {
            spheres[sphere_index].time_travel(seconds_until);
        }

        match object {
            CollisionObject::Border(i) => {
                let border = &borders[i];
                let projected_velocity = spheres[sphere_index].velocity.project_on(border.normal);
                spheres[sphere_index].velocity -= 2.0 * projected_velocity;
            }
            CollisionObject::Sphere(other_sphere_index) => {
                let cv = (spheres[other_sphere_index].position - spheres[sphere_index].position)
                    .normalize();
                let cvp = Vector2::new(cv.y, cv.x);

                let mut v1 = Vector2::new(
                    spheres[sphere_index].velocity.dot(cvp),
                    spheres[sphere_index].velocity.dot(cv),
                );
                let mut v2 = Vector2::new(
                    spheres[other_sphere_index].velocity.dot(cvp),
                    spheres[other_sphere_index].velocity.dot(cv),
                );

                let m1 = spheres[sphere_index].mass.powi(3);
                let m2 = spheres[other_sphere_index].mass.powi(3);

                let r = v2.y - v1.y;
                let i = v1.y * m1 + v2.y * m2;
                v1.y = (i + m2 * r) / (m1 + m2);
                v2.y = (i - m1 * r) / (m1 + m2);

                spheres[sphere_index].velocity = cvp * v1.x + cv * v1.y;
                spheres[other_sphere_index].velocity = cvp * v2.x + cv * v2.y;

                dbg!("boom");
            }
        }
    }

    for sphere in spheres {
        sphere.time_travel(delta_seconds);
    }
}
