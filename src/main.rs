#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use image::{ImageBuffer, RgbImage, Rgb};
use glam::Vec3A;
use glam::vec3a;

mod math;
use math::Ray;

fn ray_color(r: Ray) -> Vec3A {
    let r = r.dir.normalize();
    let t = r.y * 0.5 + 0.5;
    Vec3A::ONE.lerp(vec3a(0.5, 0.7, 1.0), t)
}

fn main() {
    let nx = 200;
    let ny = 100;
    let aspect_ratio = nx as f32 / ny as f32;

    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = vec3a(0.0, 0.0, 0.0);
    let horizontal = vec3a(viewport_width, 0.0, 0.0);
    let vertical = vec3a(0.0, viewport_height, 0.0);
    let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - vec3a(0.0, 0.0, focal_length);

    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    for j in 0..ny {
        for i in 0..nx {
            let u = i as f32 / (nx - 1) as f32;
            let v = j as f32 / (ny - 1) as f32;
            let c = ray_color(Ray{
                org: origin,
                dir: lower_left_corner + u * horizontal + v * vertical - origin,
            });

            img.put_pixel(i, j, Rgb([
                (c.x * 255.99) as u8,
                (c.y * 255.99) as u8,
                (c.z * 255.99) as u8,
            ]));
        }
    }
    image::imageops::flip_vertical_in_place(&mut img);
    img.save("test.png").unwrap();
}
