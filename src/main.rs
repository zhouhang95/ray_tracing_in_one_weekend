#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use std::sync::Arc;
use std::sync::mpsc::channel;

use image::{ImageBuffer, RgbImage, Rgb};
use glam::*;

mod math;
use math::*;

mod camera;

mod hitable;
use hitable::*;

mod material;
use material::*;

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

fn ray_color(r: Ray, world: &HitableList, light: &Arc<dyn Hitable>, depth: i32) -> Vec3A {
    assert!(vec3a_near_one(r.d));
    if depth > MAX_DEPTH {
        return Vec3A::ZERO;
    }
    let mut rec = HitRecord::default();
    if world.hit(&r, 1e-3, f32::MAX, &mut rec) {
        let mut ret = rec.mat.as_ref().unwrap().emitted(rec.uv, rec.p);
        let mut srec = ScatterRecord::default();
        if rec.mat.as_ref().unwrap().scatter(&r, &rec, &mut srec) {
            if (srec.is_spec) {
                ret += srec.attenuation * ray_color(srec.spec_ray, &world, light, depth+1);
            } else {
                let light_pdf = Arc::new(HitablePDF{ o: rec.p, ptr: light.clone() });
                let mix_pdf = MixPDF { p0: srec.pdf.unwrap().clone(), p1: light_pdf, mix: 0.5 };
                let (light_dir, direct_light) = mix_pdf.gen();
                let scattered = Ray {o: rec.p, d: light_dir, s: r.s};
                let pdf = mix_pdf.value(light_dir);
                let pdf_value = rec.mat.as_ref().unwrap().scatter_pdf(&r, &rec, &scattered);
                let mut pdf_ratio = pdf_value / pdf;
                if pdf_ratio == 0. || pdf_ratio.is_nan() {
                    return ret;
                }
                let mut contrib = srec.attenuation * ray_color(scattered, &world, light, depth+1) * pdf_ratio;
                if !direct_light {
                    contrib = contrib.min(Vec3A::splat(10.));
                }
                ret += contrib;
            }
        }
        ret
    } else {
        SKY_COLOR.get().unwrap()(r.d)
    }
}

fn main() {
    ENV_TEX.set(ImageTex::new("res/newport_loft.jpg".into())).unwrap();
    let samples_per_pixel = 1024;

    let nx = 400;
    let ny = 400;
    let aspect_ratio = nx as f32 / ny as f32;

    let t = EZTimer::new();

    let (tx, rx) = channel();
    let pool = threadpool::Builder::new().build();
    let (world, lights, cam) = book3(aspect_ratio);

    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    for i in 0..nx {
        let tx = tx.clone();
        let world = world.clone();
        let light = lights.clone();
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
                    c += ray_color(r, &world, &light, 0);
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
