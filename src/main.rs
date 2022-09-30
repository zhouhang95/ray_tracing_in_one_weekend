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

mod utils;
use utils::*;

use rand::Rng;

use chrono::prelude::*;

fn ray_color(r: Ray, world: &HitableList, depth: i32) -> Vec3A {
    if depth <= 0 {
        return vec3a(0.0, 0.0, 0.0);
    }
    let mut rec = HitRecord::default();
    if world.hit(&r, 1e-3, f32::MAX, &mut rec) {
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
    let samples_per_pixel = 32;
    let max_depth = 50;

    let nx = 400;
    let ny = 200;
    let aspect_ratio = nx as f32 / ny as f32;

    let material_ground = Arc::new(Lambertian { albedo: vec3a(0.5, 0.5, 0.5)});
    let material_1 = Arc::new(Lambertian { albedo: vec3a(0.1, 0.2, 0.5)});
    let material_2 = Arc::new(Dielectric {ior : 1.5});
    let material_3 = Arc::new(Metal { albedo: vec3a(0.8, 0.6, 0.2), fuzz: 0.});

    let mut world: HitableList = vec![
        Arc::new(Sphere {c: vec3a( 0.0, -1000., -1.0), r: 1000.0, mat: material_ground}),
        Arc::new(Sphere {c: vec3a( 0.0, 1.0, 0.0), r: 1., mat: material_1}),
        Arc::new(Sphere {c: vec3a(-4.0, 1.0, 0.0), r: 1., mat: material_2.clone()}),
        Arc::new(Sphere {c: vec3a( 4.0, 1.0, 0.0), r: 1., mat: material_3}),
    ];

    let mut rng = rand::thread_rng();

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat = rng.gen::<f32>();
            let center = vec3a(a as f32 + 0.9*rng.gen::<f32>(), 0.2, b as f32 + 0.9*rng.gen::<f32>());
            let mat: Arc<dyn Material> = if choose_mat < 0.8 {
                let albedo = vec3a_random() * vec3a_random();
                Arc::new(Lambertian { albedo})
            } else if choose_mat < 0.95 {
                let albedo = vec3a_random_range(0.5, 1.);
                let fuzz = rng.gen::<f32>();
                Arc::new(Metal {albedo, fuzz})
            } else {
                material_2.clone()
            };
            world.push(Arc::new(Sphere {c: center, r: 0.2, mat}));

        }
    }

    let cam = Camera::new(
        vec3a(13., 2., 3.),
        vec3a(0., 0., 0.),
        vec3a(0., 1., 0.),
        20.,
        aspect_ratio,
    );
    let t = EZTimer::new();

    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    for j in 0..ny {
        eprintln!("{}/{}", j, ny);
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
    drop(t);
    image::imageops::flip_vertical_in_place(&mut img);
    let local = Local::now().to_rfc3339().replace(":", "-");
    let datetime = local.split_once(".").unwrap().0;
    let file_name = format!("{}.png", datetime);
    img.save(file_name).unwrap();
}
