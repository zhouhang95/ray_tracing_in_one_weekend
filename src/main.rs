#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use image::{ImageBuffer, RgbImage, Rgb};
use glam::Vec3A;
use glam::vec3a;

mod math;
use math::*;

mod camera;
use camera::Camera;

use rand::Rng;

fn ray_color(r: Ray, world: &HitableList) -> Vec3A {
    let mut rec = HitRecord::default();
    if world.hit(&r, 0.0, f32::MAX, &mut rec) {
        rec.norm * 0.5 + 0.5
    } else {
        let t = r.d.y * 0.5 + 0.5;
        Vec3A::ONE.lerp(vec3a(0.5, 0.7, 1.0), t)
    }
}

fn main() {
    let samples_per_pixel = 128;

    let nx = 200;
    let ny = 100;
    let aspect_ratio = nx as f32 / ny as f32;

    let world: HitableList = vec![
        Box::new(Sphere {c: vec3a(0.0, 0.0, -1.0), r: 0.5}),
        Box::new(Sphere {c: vec3a(0.0, -100.5, -1.0), r: 100.0}),
    ];

    let cam = Camera::new(aspect_ratio);

    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    let mut rng = rand::thread_rng();
    for j in 0..ny {
        for i in 0..nx {
            let mut c = Vec3A::ZERO;
            for _ in 0..samples_per_pixel {
                let u = (i as f32 + rng.gen::<f32>()) / nx as f32;
                let v = (j as f32 + rng.gen::<f32>()) / ny as f32;
                let r = cam.get_ray(u, v);
                c += ray_color(r, &world);
            }
            c /= samples_per_pixel as f32;


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
