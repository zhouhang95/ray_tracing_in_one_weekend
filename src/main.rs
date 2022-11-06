#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use std::sync::mpsc::channel;

use image::{ImageBuffer, RgbImage, Rgb};
use glam::Vec3A;

mod math;
use math::*;

mod camera;

mod hitable;
use hitable::*;

mod material;

mod utils;
use utils::*;

mod texture;
use texture::*;

mod lib;
use lib::*;

mod demo_scene;
use demo_scene::*;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use chrono::prelude::*;

const MAX_DEPTH: i32 = 50;

fn ray_color(r: Ray, world: &HitableList, depth: i32) -> Vec3A {
    assert!(vec3a_near_one(r.d));
    if depth > MAX_DEPTH {
        return Vec3A::ZERO;
    }
    let mut rec = HitRecord::default();
    if world.hit(&r, 1e-3, f32::MAX, &mut rec) {
        let mut scattered = Ray {o: Vec3A::ZERO, d: Vec3A::ZERO, s: r.s};
        let mut attenuation = Vec3A::ONE;
        let mut pdf = 0.;
        let mut ret = rec.mat.as_ref().unwrap().emitted(rec.uv, rec.p);
        if rec.mat.as_ref().unwrap().scatter(&r, &rec, &mut attenuation, &mut scattered, &mut pdf) {
            // let russian_roulette = RNG.with(|rng| rng.borrow_mut().gen::<f32>());
            // let threshold = attenuation.max_element();
            // if russian_roulette < threshold {
            //     ret += attenuation * ray_color(scattered, &world, depth+1) / threshold;
            // }
            ret += attenuation * ray_color(scattered, &world, depth+1) / pdf * rec.mat.as_ref().unwrap().scatter_pdf(&r, &rec, &scattered);
        }
        ret
    } else {
        SKY_COLOR.get().unwrap()(r.d)
    }
}

fn main() {
    ENV_TEX.set(ImageTex::new("res/newport_loft.jpg".into())).unwrap();
    let samples_per_pixel = 128;

    let nx = 400;
    let ny = 400;
    let aspect_ratio = nx as f32 / ny as f32;

    let t = EZTimer::new();

    let (tx, rx) = channel();
    let pool = threadpool::Builder::new().build();
    let (world, cam) = book3(aspect_ratio);

    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    for i in 0..nx {
        let tx = tx.clone();
        let world = world.clone();
        pool.execute(move || {
            RNG.with(|rng| {
                *rng.borrow_mut() = SmallRng::seed_from_u64(95 + i as u64);
            });
            for j in 0..ny {
                let mut c = Vec3A::ZERO;
                let mut rays: Vec<Ray> = Vec::with_capacity(samples_per_pixel);
                RNG.with(|rng| {
                    for _ in 0..samples_per_pixel {
                        let u = (i as f32 + rng.borrow_mut().gen::<f32>()) / nx as f32;
                        let v = (j as f32 + rng.borrow_mut().gen::<f32>()) / ny as f32;
                        let r = cam.get_ray(u, v);
                        rays.push(r);
                    }
                });
                for r in rays {
                    c += ray_color(r, &world, 0);
                }
                c /= samples_per_pixel as f32;
                c = c.powf(1.0 / 2.0);

                tx.send((i, j, Rgb([
                    (c.x * 255.99) as u8,
                    (c.y * 255.99) as u8,
                    (c.z * 255.99) as u8,
                ]))).unwrap();
            }
        });
    }
    drop(tx);
    let local = Local::now().to_rfc3339().replace(":", "-");
    let datetime = local.split_once(".").unwrap().0;
    let file_name = format!("{}.png", datetime);
    let mut count = 0;
    while let Ok((i, j, col)) = rx.recv() {
        count += 1;
        if count % (nx * 10) == 0 {
            eprintln!("{}/{}", count / nx, ny);
        }
        img.put_pixel(i, j, col);
        if count % (nx * 10) == 0{
            let mut img = img.clone();
            image::imageops::flip_vertical_in_place(&mut img);
            img.save(file_name.clone()).unwrap();
        }
    }
    drop(t);
    image::imageops::flip_vertical_in_place(&mut img);
    img.save(file_name).unwrap();
}
