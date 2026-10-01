use cgmath::{Vector2, prelude::*};
use std::time::Duration;

type Mass = f64;
type Velocity = f64;
type Position = f64;
type Length = f64;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub position: Vector2<Position>,
    pub velocity: Vector2<Velocity>,
    pub radius: Length,
}

impl Sphere {
    fn mass(&self) -> Mass {
        self.radius.powi(3)
    }
}

pub struct Border {
    pub normal: Vector2<Length>,
    pub position: Vector2<Length>,
}

pub fn collides_with_border(sphere: &Sphere, border: &Border) -> bool {
    let distance = (sphere.position - border.position).dot(border.normal.normalize());

    distance < sphere.radius
}

pub fn perpendicular(v: Vector2<Velocity>) -> Vector2<Velocity> {
    Vector2::new(v.y, v.x)
}

pub fn collide_with_border(sphere: &mut Sphere, border: &Border) {
    let u = sphere.velocity.dot(border.normal.normalize());

    sphere.position = sphere.position.project_on(perpendicular(border.normal))
        + border.position
        + border.normal * sphere.radius;

    let v = -u;

    sphere.velocity -= u * border.normal.normalize();
    sphere.velocity += v * border.normal.normalize();
}

pub fn collides_with_sphere(sphere1: &Sphere, sphere2: &Sphere, delta_seconds: f64) -> bool {
    (sphere1.position + sphere1.velocity * delta_seconds)
        .distance(sphere2.position + sphere2.velocity * delta_seconds)
        < sphere1.radius + sphere2.radius
}

pub fn separate_spheres(sphere1: &mut Sphere, sphere2: &mut Sphere) {
    let cv = sphere2.position - sphere1.position;

    sphere1.position -= cv.normalize() * (sphere1.radius + sphere2.radius - cv.magnitude()) / 2.0;
    sphere2.position += cv.normalize() * (sphere1.radius + sphere2.radius - cv.magnitude()) / 2.0;
}

pub fn collide_with_sphere(sphere1: &mut Sphere, sphere2: &mut Sphere) {
    separate_spheres(sphere1, sphere2);

    let cv = sphere2.position - sphere1.position;

    let u1 = sphere1.velocity.dot(cv.normalize());
    let u2 = sphere2.velocity.dot(cv.normalize());

    // Remove paralell velocity
    sphere1.velocity -= u1 * cv.normalize();
    sphere2.velocity -= u2 * cv.normalize();

    let m1 = sphere1.mass();
    let m2 = sphere2.mass();

    let (v1, v2) = collide_with_sphere_paralell(m1, m2, u1, u2);

    // Add new paralell velocity
    sphere1.velocity += v1 * cv.normalize();
    sphere2.velocity += v2 * cv.normalize();
}

pub fn tick(
    spheres: &mut Vec<Sphere>,
    borders: &[Border],
    delta_duration: Duration,
    energies: &mut Vec<f64>,
) -> bool {
    let delta_seconds = delta_duration.as_secs_f64();
    let mut b = false;

    let mut energy = 0.0;

    for i in 0..spheres.len() {
        for j in (i + 1)..spheres.len() {
            let [sphere1, sphere2] = spheres.get_disjoint_mut([i, j]).unwrap();

            if collides_with_sphere(sphere1, sphere2, delta_seconds) {
                dbg!(energies.iter().sum::<f64>() / energies.len() as f64);
                collide_with_sphere(sphere1, sphere2);
                b = true;
            }
        }
    }

    for i in 0..spheres.len() {
        for border in borders {
            let sphere = spheres.get_mut(i).unwrap();

            if collides_with_border(sphere, border) {
                collide_with_border(sphere, border);
            }
        }
    }

    for sphere in spheres.iter_mut() {
        // sphere.velocity.y -= 9.82 * delta_seconds / 2.0;
        sphere.position += sphere.velocity * delta_seconds;
        // sphere.velocity.y -= 9.82 * delta_seconds / 2.0;

        energy += sphere.mass() * sphere.velocity.magnitude2() / 2.0;
    }

    energies.push(energy);

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
