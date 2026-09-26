use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub x: f32,
    pub y: f32,
    pub xv: f32,
    pub yv: f32,
    pub r: f32,
}

pub fn tick(
    spheres: &mut Vec<Sphere>,
    screen_width: f32,
    screen_height: f32,
    delta_time: Duration,
) {
    let delta_time = delta_time.as_secs_f32();
    for sphere in spheres.iter_mut() {
        sphere.yv -= 9.82 * delta_time / 2.0;
        sphere.x += sphere.xv * delta_time;
        sphere.y += sphere.yv * delta_time;
        sphere.yv -= 9.82 * delta_time / 2.0;

        if sphere.y < sphere.r {
            sphere.y = sphere.r;
            sphere.yv = sphere.yv.abs();
        }
        if sphere.y > screen_height - sphere.r {
            sphere.y = screen_height - sphere.r;
            sphere.yv = -sphere.yv.abs();
        }
        if sphere.x > screen_width - sphere.r {
            sphere.x = screen_width - sphere.r;
            sphere.xv = -sphere.xv.abs();
        }
        if sphere.x < sphere.r {
            sphere.x = sphere.r;
            sphere.xv = sphere.xv.abs();
        }
    }
}
