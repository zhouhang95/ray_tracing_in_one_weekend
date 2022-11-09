use std::sync::Arc;

use crate::camera::Camera;
use crate::hitable::*;
use crate::material::*;
use crate::texture::*;
use crate::math::*;
use once_cell::sync::OnceCell;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use glam::*;

use crate::SKY_COLOR;
use crate::RNG;

pub static ENV_TEX: OnceCell<ImageTex> = OnceCell::new();

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

fn build_bvh(world: &mut Vec<Arc<dyn Hitable>>) -> Vec<Arc<dyn Hitable>> {
    let world_len = world.len();
    let bvh: Arc<dyn Hitable> = Arc::new(BvhNode::new(world, 0, world_len));
    vec![bvh]
}

pub fn book3(aspect_ratio: f32) -> (Vec<Arc<dyn Hitable>>, Arc<dyn Hitable>, Camera) {
    SKY_COLOR.set(black_sky).unwrap();

    let red = Arc::new(Diffuse { albedo: Arc::new(ConstantTex{ col: vec3a(0.65, 0.05, 0.05)})});
    let white = Arc::new(Diffuse { albedo: Arc::new(ConstantTex{ col: vec3a(0.73, 0.73, 0.73)})});
    let green = Arc::new(Diffuse { albedo: Arc::new(ConstantTex{ col: vec3a(0.12, 0.45, 0.15)})});
    let light = Arc::new(Emission { emit: Arc::new(ConstantTex{ col: vec3a(15., 15., 15.)})});
    let metal = Arc::new(Metal { albedo: vec3a(0.8, 0.8, 0.8), fuzz: 0. });
    let glass = Arc::new( Dielectric { ior: 1.5 });

    let sphere = Arc::new(Sphere{c: vec3a(190., 90., 190.), r: 90., mat: glass, name: "".to_string()});
    let box_1 = Arc::new(GBox::new(Vec3A::ZERO, vec3a(165., 330., 165.), metal));
    let box_1 = Arc::new(RotateY::new(box_1, 15.));
    let box_1 = Arc::new(Translate {offset: vec3a(265., 0., 295.), ptr: box_1});
    let box_2 = Arc::new(GBox::new(Vec3A::ZERO, vec3a(165., 165., 165.), white.clone()));
    let box_2 = Arc::new(RotateY::new(box_2, -18.));
    let box_2 = Arc::new(Translate {offset: vec3a(130., 0., 65.), ptr: box_2});

    let ligth_obj = Arc::new(XZRect {min: vec3a(213., 554., 227.), max: vec3a(343., 554., 332.), mat: light.clone()});

    let mut world: HitableList = vec![
        ligth_obj.clone(),
        Arc::new(XYRect {min: vec3a(0., 0., 555.), max: vec3a(555., 555., 555.), mat: white.clone()}),
        Arc::new(XZRect {min: vec3a(0., 0., 0.), max: vec3a(555., 0., 555.), mat: white.clone()}),
        Arc::new(XZRect {min: vec3a(0., 555., 0.), max: vec3a(555., 555., 555.), mat: white.clone()}),
        Arc::new(YZRect {min: vec3a(0., 0., 0.), max: vec3a(0., 555., 555.), mat: red.clone()}),
        Arc::new(YZRect {min: vec3a(555., 0., 0.), max: vec3a(555., 555., 555.), mat: green.clone()}),
        box_1,
        sphere,
    ];
    let cam = Camera::new(
        vec3a(278., 278., -800.),
        vec3a(278., 278., 0.),
        vec3a(0., 1., 0.),
        40.,
        aspect_ratio,
    );
    (build_bvh(&mut world), ligth_obj, cam)
}

