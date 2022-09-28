#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use std::sync::Arc;

use image::{ImageBuffer, RgbImage, Rgb};
use glam::Vec3A;
use glam::vec3a;

mod math;
use math::*;

mod camera;
use camera::Camera;

mod hitable;
use hitable::*;

mod material;
use material::*;

use rand::Rng;

use chrono::prelude::*;

fn ray_color(r: Ray, world: &HitableList, depth: i32) -> Vec3A {
    if depth <= 0 {
        return vec3a(0.0, 0.0, 0.0);
    }
    let mut rec = HitRecord::default();
    if world.hit(&r, 1e-4, f32::MAX, &mut rec) {
        let mut scattered = Ray {o: Vec3A::ZERO, d: Vec3A::ZERO};
        let mut attenuation = Vec3A::ZERO;
        if rec.mat.as_ref().unwrap().scatter(&r, &rec, &mut attenuation, &mut scattered) {
            attenuation * ray_color(scattered, &world, depth-1)
        } else {
            Vec3A::ZERO
        }
    } else {
        let t = r.d.y * 0.5 + 0.5;
        Vec3A::ONE.lerp(vec3a(0.5, 0.7, 1.0), t)
    }
}

fn main() {
    let samples_per_pixel = 512;
    let max_depth = 50;

    let nx = 400;
    let ny = 200;
    let aspect_ratio = nx as f32 / ny as f32;

    let material_ground = Arc::new(Lambertian { albedo: vec3a(0.8, 0.8, 0.0)});
    let material_center = Arc::new(Lambertian { albedo: vec3a(0.1, 0.2, 0.5)});
    let material_left = Arc::new(Dielectric {ior : 1.5});
    let material_right = Arc::new(Metal { albedo: vec3a(0.8, 0.6, 0.2), fuzz: 0.});

    let world: HitableList = vec![
        Arc::new(Sphere {c: vec3a( 0.0, -100.5, -1.0), r: 100.0, mat: material_ground}),
        Arc::new(Sphere {c: vec3a( 0.0, 0.0, -1.0), r: 0.5, mat: material_center}),
        Arc::new(Sphere {c: vec3a(-1.0, 0.0, -1.0), r: 0.5, mat: material_left.clone()}),
        Arc::new(Sphere {c: vec3a(-1.0, 0.0, -1.0), r: -0.4, mat: material_left}),
        Arc::new(Sphere {c: vec3a( 1.0, 0.0, -1.0), r: 0.5, mat: material_right}),
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
                c += ray_color(r, &world, max_depth);
            }
            c /= samples_per_pixel as f32;
            c = c.powf(1.0 / 2.0);

            img.put_pixel(i, j, Rgb([
                (c.x * 255.99) as u8,
                (c.y * 255.99) as u8,
                (c.z * 255.99) as u8,
            ]));
        }
    }
    image::imageops::flip_vertical_in_place(&mut img);
    let local = Local::now().to_rfc3339().replace(":", "-");
    let datetime = local.split_once(".").unwrap().0;
    let file_name = format!("{}.png", datetime);
    img.save(file_name).unwrap();
}
