use cgmath::{Vector2, prelude::*};
use std::time::Duration;

type Mass = f32;
type Velocity = f32;
type Position = f32;
type Length = f32;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub position: Vector2<Position>,
    pub velocity: Vector2<Velocity>,
    pub radius: Length,
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

    sphere.position = sphere.position.project_on(perpendicular(border.normal))
        + border.position
        + border.normal * sphere.radius;

    sphere.velocity = paralell_velocity - perpendicular_velocity;
}

pub fn collides_with_sphere(sphere1: &Sphere, sphere2: &Sphere, delta_seconds: f32) -> bool {
    (sphere1.position + sphere1.velocity * delta_seconds)
        .distance(sphere2.position + sphere2.velocity * delta_seconds)
        < sphere1.radius + sphere2.radius
}

pub fn separate_spheres(sphere1: &mut Sphere, sphere2: &mut Sphere) {
    let cv = sphere2.position - sphere1.position;

    sphere1.position += -cv.normalize() * (sphere1.radius + sphere2.radius - cv.magnitude())
}

pub fn collide_with_sphere(sphere1: &mut Sphere, sphere2: &mut Sphere) {
    separate_spheres(sphere1, sphere2);

    let cv = sphere2.position - sphere1.position;

    let u1 = sphere1.velocity.dot(cv.normalize());
    let u2 = sphere2.velocity.dot(cv.normalize());

    // Remove paralell velocity
    sphere1.velocity -= u1 * cv.normalize();
    sphere2.velocity -= u2 * cv.normalize();

    let m1 = sphere1.radius;
    let m2 = sphere2.radius;

    let (v1, v2) = collide_with_sphere_paralell(m1, m2, u1, u2);

    // Add new paralell velocity
    sphere1.velocity += v1 * cv.normalize();
    sphere2.velocity += v2 * cv.normalize();
}

pub fn tick(spheres: &mut Vec<Sphere>, borders: &[Border], delta_duration: Duration) -> bool {
    let delta_seconds = delta_duration.as_secs_f32();
    let mut b = false;

    for sphere in spheres.iter_mut() {
        sphere.position += sphere.velocity * delta_seconds;
        sphere.velocity.y -= 9.82;
    }

    for i in 0..spheres.len() {
        for border in borders {
            let sphere = spheres.get_mut(i).unwrap();

            if collides_with_border(sphere, border) {
                collide_with_border(sphere, border);
            }
        }

        for j in (i + 1)..spheres.len() {
            let [sphere1, sphere2] = spheres.get_disjoint_mut([i, j]).unwrap();

            if collides_with_sphere(sphere1, sphere2, delta_seconds) {
                collide_with_sphere(sphere1, sphere2);
                b = true;
            }
        }
    }

    return b;
}

fn collide_with_sphere_paralell(
    m1: Mass,
    m2: Mass,
    u1: Velocity,
    u2: Velocity,
) -> (Velocity, Velocity) {
    let inertia = m1 * u1 + m2 * u2;
    let relative_velocity = u2 - u1;

    let v1 = (inertia + m2 * relative_velocity) / (m1 + m2);
    let v2 = v1 - relative_velocity;

    dbg!(u1, u2, v1, v2);
    (v1, v2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paralell_collision_same_mass() {
        assert_eq!(
            collide_with_sphere_paralell(1.0, 1.0, 1.0, -1.0),
            (-1.0, 1.0)
        );
    }

    #[test]
    fn same_direction_1d_collision() {
        assert_eq!(collide_with_sphere_paralell(1.0, 1.0, 2.0, 1.0), (1.0, 2.0));
    }

    #[test]
    fn same_direction_1d_collision_different_masses() {
        let (v1, v2) = collide_with_sphere_paralell(10.0, 1.0, 2.0, 1.0);

        assert!(v2 > v1);
        assert!(v2 > 0.0);
        assert!(v1 > 0.0);
    }

    #[test]
    fn seprate_spheres() {
        let s = |x, y| Sphere {
            position: Vector2::new(x, y),
            velocity: Vector2::zero(),
            radius: 100.0,
        };

        let mut s1 = s(0.0, 0.0);
        let mut s2 = s(50.0, 0.0);

        separate_spheres(&mut s1, &mut s2);

        assert!(s1.position.distance(s2.position) >= 200.0);

        let mut s1 = s(0.0, 0.0);
        let mut s2 = s(100.0, 100.0);

        separate_spheres(&mut s1, &mut s2);

        assert!(s1.position.distance(s2.position) >= 200.0);
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
