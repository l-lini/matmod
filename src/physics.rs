use cgmath::{Vector2, prelude::*};
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub radius: f32,
}

pub struct Border {
    pub normal: Vector2<f32>,
    pub position: Vector2<f32>,
}

pub fn collides_with_border(sphere: &Sphere, border: &Border) -> bool {
    let distance = (sphere.position - border.position).dot(border.normal.normalize());

    distance < sphere.radius
}

pub fn perpendicular(v: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(v.y, v.x)
}

pub fn collide_with_border(sphere: &mut Sphere, border: &Border) {
    let paralell_velocity = sphere.velocity.project_on(perpendicular(border.normal));
    let perpendicular_velocity = sphere.velocity.project_on(border.normal);

    sphere.velocity = paralell_velocity - perpendicular_velocity;
}

pub fn collides_with_sphere(sphere1: &Sphere, sphere2: &Sphere) -> bool {
    sphere1.position.distance(sphere2.position) < sphere1.radius + sphere2.radius
}

pub fn collide_paralell(m1: f32, m2: f32, v1: f32, v2: f32) -> (f32, f32) {
    dbg!(v1, v2);
    (v2, v1)
}

pub fn collide_with_sphere(sphere1: &mut Sphere, sphere2: &mut Sphere) {
    let collision_vector = (sphere1.position - sphere2.position).normalize();

    let u1 = sphere1.velocity.dot(collision_vector);
    let u2 = sphere2.velocity.dot(collision_vector);

    let sphere1_perpendicular_velocity =
        sphere1.velocity.project_on(perpendicular(collision_vector));
    let sphere2_perpendicular_velocity =
        sphere2.velocity.project_on(perpendicular(collision_vector));

    let (v1, v2) = (u2, u1);

    sphere1.velocity = v1 * collision_vector + sphere1_perpendicular_velocity;
    sphere2.velocity = v2 * collision_vector + sphere2_perpendicular_velocity;
}

pub fn tick(spheres: &mut Vec<Sphere>, borders: &[Border], delta_duration: Duration) -> bool {
    let mut b = false;
    for i in 0..spheres.len() {
        for border in borders {
            let sphere = spheres.get_mut(i).unwrap();

            if collides_with_border(sphere, border) {
                collide_with_border(sphere, border);
            }
        }

        for j in (i + 1)..spheres.len() {
            dbg!(i, j);
            let [sphere1, sphere2] = spheres.get_disjoint_mut([i, j]).unwrap();

            if collides_with_sphere(sphere1, sphere2) {
                collide_with_sphere(sphere1, sphere2);
                b = true;
            }
        }
    }

    let delta_seconds = delta_duration.as_secs_f32();
    for sphere in spheres {
        sphere.position += sphere.velocity * delta_seconds;
        // sphere.velocity.y -= 9.82;
    }

    return b;

    dbg!(delta_seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paralell_collision_same_mass() {
        let m1 = 1.0;
        let m2 = m1;
        let v1 = 99.0;
        let v2 = -50.0;
        assert_eq!(collide_paralell(m1, m2, v1, v2), (v2, v1));
    }

    #[test]
    fn collides_with_border_inside_border() {
        let b = Border {
            normal: Vector2::new(-1.0, 0.0),
            position: Vector2::zero(),
        };
        let s = Sphere {
            position: Vector2::zero(),
            velocity: Vector2::zero(),
            radius: 1.0,
        };

        assert!(collides_with_border(&s, &b));
    }
}
