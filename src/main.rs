#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]

use std::sync::Arc;
use std::sync::mpsc::channel;

use glam::vec2;
use image::{ImageBuffer, RgbImage, Rgb};
use glam::Vec3A;
use glam::vec3a;
use once_cell::sync::OnceCell;

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

mod texture;
use texture::*;

mod lib;
use lib::*;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use chrono::prelude::*;

static ENV_TEX: OnceCell<ImageTex> = OnceCell::new();
static SKY_COLOR: OnceCell<fn(Vec3A) -> Vec3A> = OnceCell::new();

#[allow(dead_code)]
fn tex_sky_color(d: Vec3A) -> Vec3A {
    let uv = Sphere::get_uv(d);
    let c = ENV_TEX.get().unwrap().value(vec2(1. - uv.x, uv.y), Vec3A::ZERO);
    c * c
}

fn sky_color(d: Vec3A) -> Vec3A {
    let t = d.y * 0.5 + 0.5;
    Vec3A::ONE.lerp(vec3a(0.5, 0.7, 1.0), t)
}

fn black_sky(_d: Vec3A) -> Vec3A {
    Vec3A::ZERO
}

fn ray_color(r: Ray, world: &HitableList, depth: i32) -> Vec3A {
    if depth <= 0 {
        return Vec3A::ZERO;
    }
    let mut rec = HitRecord::default();
    if world.hit(&r, 1e-3, f32::MAX, &mut rec) {
        let mut scattered = Ray {o: Vec3A::ZERO, d: Vec3A::ZERO, s: r.s};
        let mut attenuation = Vec3A::ZERO;
        let emit = rec.mat.as_ref().unwrap().emitted(rec.uv, rec.p);
        if rec.mat.as_ref().unwrap().scatter(&r, &rec, &mut attenuation, &mut scattered) {
            emit + attenuation * ray_color(scattered, &world, depth-1)
        } else {
            emit
        }
    } else {
        SKY_COLOR.get().unwrap()(r.d)
    }
}

fn sphere_scene(aspect_ratio: f32) -> (Vec<Arc<dyn Hitable>>, Camera) {
    SKY_COLOR.set(sky_color).unwrap();

    // let checker = Arc::new(CheckerTex::new(vec3a(0.2, 0.3, 0.1), vec3a(0.9, 0.9, 0.9)));
    let perlin = Arc::new(PerlinTex::new(4.));
    let earth_map = Arc::new(ImageTex::new("res/earthmap.jpg".into()));

    let material_ground = Arc::new(Lambertian { albedo: perlin});
    let material_1 = Arc::new(Emission { emit: earth_map});
    let material_2 = Arc::new(Dielectric {ior : 1.5});
    let material_3 = Arc::new(Metal { albedo: vec3a(0.8, 0.6, 0.2), fuzz: 0.});

    let mut world: HitableList = vec![
        Arc::new(Sphere {c: vec3a( 1.0, -1000., -1.0), r: 1000.0, mat: material_ground, name: "Ground".to_string()}),
        Arc::new(Sphere {c: vec3a( 0.0, 1.0, 3.0), r: 1., mat: material_1, name: "Sphere_1".to_string()}),
        Arc::new(Sphere {c: vec3a(-4.0, 1.0, 0.0), r: 1., mat: material_2.clone(), name: "Sphere_2".to_string()}),
        Arc::new(Sphere {c: vec3a( 4.0, 1.0, 0.0), r: 1., mat: material_3, name: "Sphere_3".to_string()}),
    ];

    let mut rng = SmallRng::seed_from_u64(95);

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat = rng.gen::<f32>();
            let center = vec3a(a as f32 + 0.9*rng.gen::<f32>(), 0.2, b as f32 + 0.9*rng.gen::<f32>());
            let mat: Arc<dyn Material> = if choose_mat < 0.8 {
                let a = vec3a(rng.gen(), rng.gen(), rng.gen());
                let b = vec3a(rng.gen(), rng.gen(), rng.gen());
                let col =  a * b;
                let albedo = Arc::new(ConstantTex {col});
                Arc::new(Lambertian {albedo})
            } else if choose_mat < 0.95 {
                let albedo = vec3a(rng.gen(), rng.gen(), rng.gen()) * 0.5 + 0.5;
                let fuzz = rng.gen::<f32>();
                Arc::new(Metal {albedo, fuzz})
            } else {
                material_2.clone()
            };
            world.push(Arc::new(Sphere {c: center, r: 0.2, mat, name: format!("Sphere {}, {}", a, b)}));
        }
    }
    let cam = Camera::new(
        vec3a(13., 2., 3.),
        vec3a(0., 0., 0.),
        vec3a(0., 1., 0.),
        20.,
        aspect_ratio,
    );
    (build_bvh(&mut world), cam)
}

fn simple_light_scene(aspect_ratio: f32) -> (Vec<Arc<dyn Hitable>>, Camera) {
    SKY_COLOR.set(black_sky).unwrap();

    let perlin = Arc::new(PerlinTex::new(4.));

    let mat_perlin = Arc::new(Lambertian { albedo: perlin});
    let material_1 = Arc::new(Emission { emit: Arc::new(ConstantTex{ col: vec3a(4., 4., 4.)})});

    let mut world: HitableList = vec![
        Arc::new(Sphere {c: vec3a( 1.0, -1000., -1.0), r: 1000.0, mat: mat_perlin.clone(), name: "Ground".to_string()}),
        Arc::new(Sphere {c: vec3a( 0.0, 2.0, 0.0), r: 2., mat: mat_perlin.clone(), name: "Sphere_1".to_string()}),
        Arc::new(Sphere {c: vec3a( 0.0, 6.5, 0.0), r: 2., mat: material_1.clone(), name: "Sphere_2".to_string()}),
        Arc::new(XYRect {min: vec3a(3., 1., -2.), max: vec3a(5., 3., -2.), mat: material_1.clone()})
    ];
    let cam = Camera::new(
        vec3a(26., 3., 6.),
        vec3a(0., 0., 0.),
        vec3a(0., 2., 0.),
        20.,
        aspect_ratio,
    );
    (build_bvh(&mut world), cam)
}

fn cornell_box(aspect_ratio: f32) -> (Vec<Arc<dyn Hitable>>, Camera) {
    SKY_COLOR.set(black_sky).unwrap();

    let red = Arc::new(Lambertian { albedo: Arc::new(ConstantTex{ col: vec3a(0.65, 0.05, 0.05)})});
    let white = Arc::new(Lambertian { albedo: Arc::new(ConstantTex{ col: vec3a(0.73, 0.73, 0.73)})});
    let green = Arc::new(Lambertian { albedo: Arc::new(ConstantTex{ col: vec3a(0.12, 0.45, 0.15)})});
    let light = Arc::new(Emission { emit: Arc::new(ConstantTex{ col: vec3a(7., 7., 7.)})});

    let box_1 = Arc::new(GBox::new(Vec3A::ZERO, vec3a(165., 330., 165.), white.clone()));
    let box_1 = Arc::new(RotateY::new(box_1, 15.));
    let box_1 = Arc::new(Translate {offset: vec3a(265., 0., 295.), ptr: box_1});

    let box_2 = Arc::new(GBox::new(Vec3A::ZERO, vec3a(165., 165., 165.), white.clone()));
    let box_2 = Arc::new(RotateY::new(box_2, -18.));
    let box_2 = Arc::new(Translate {offset: vec3a(130., 0., 65.), ptr: box_2});
    let mediun_2 = Arc::new(ConstantMedium::new(box_2.clone(), 0.01, Arc::new(ConstantTex{ col: Vec3A::ONE })));

    let mut world: HitableList = vec![
        Arc::new(XZRect {min: vec3a(113., 554., 127.), max: vec3a(443., 554., 432.), mat: light.clone()}),
        Arc::new(XYRect {min: vec3a(0., 0., 555.), max: vec3a(555., 555., 555.), mat: white.clone()}),
        Arc::new(XZRect {min: vec3a(0., 0., 0.), max: vec3a(555., 0., 555.), mat: white.clone()}),
        Arc::new(XZRect {min: vec3a(0., 555., 0.), max: vec3a(555., 555., 555.), mat: white.clone()}),
        Arc::new(YZRect {min: vec3a(0., 0., 0.), max: vec3a(0., 555., 555.), mat: red.clone()}),
        Arc::new(YZRect {min: vec3a(555., 0., 0.), max: vec3a(555., 555., 555.), mat: green.clone()}),
        // box_1,
        // box_2,
        mediun_2,
    ];
    let cam = Camera::new(
        vec3a(278., 278., -800.),
        vec3a(278., 278., 0.),
        vec3a(0., 2., 0.),
        40.,
        aspect_ratio,
    );
    (build_bvh(&mut world), cam)
}

fn build_bvh(world: &mut Vec<Arc<dyn Hitable>>) -> Vec<Arc<dyn Hitable>> {
    let world_len = world.len();
    let bvh: Arc<dyn Hitable> = Arc::new(BvhNode::new(world, 0, world_len));
    vec![bvh]
}

fn main() {
    ENV_TEX.set(ImageTex::new("res/newport_loft.jpg".into())).unwrap();
    let samples_per_pixel = 32;
    let max_depth = 50;

    let nx = 800;
    let ny = 400;
    let aspect_ratio = nx as f32 / ny as f32;

    let t = EZTimer::new();

    let (tx, rx) = channel();
    let pool = threadpool::Builder::new().build();
    // let pool = threadpool::ThreadPool::new(1);
    let (world, cam) = cornell_box(aspect_ratio);

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
                    c += ray_color(r, &world, max_depth);
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
