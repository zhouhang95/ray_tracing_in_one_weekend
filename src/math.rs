use std::mem::transmute;

use glam::*;
use rand::Rng;
use std::mem::transmute;

use crate::lib::RNG;

pub fn vec3a_near_zero(v: Vec3A) -> bool {
    let s = f32::EPSILON;
    return (v.x.abs() < s) && (v.y.abs() < s) && (v.z.abs() < s);
}

pub fn vec3a_random() -> Vec3A {
    RNG.with(|rng| {
        let x = rng.borrow_mut().gen::<f32>();
        let y = rng.borrow_mut().gen::<f32>();
        let z = rng.borrow_mut().gen::<f32>();
        vec3a(x, y, z)
    })
}

pub fn vec3a_random_range(min: f32, max: f32) -> Vec3A{
    vec3a_random() * (max - min) + min
}

pub fn random_in_unit_sphere() -> Vec3A {
    loop {
        let v = vec3a_random_range(-1., 1.);
        if v.length_squared() < 1.0 {
            return v;
        }
    }
}

#[allow(dead_code)]
pub fn random_in_hemisphere(norm: Vec3A) -> Vec3A {
    let v = random_in_unit_sphere();
    if v.dot(norm) > 0.0 {
        v
    } else {
        -v
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Vec3A,
    pub d: Vec3A,
    pub s: Vec2,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.o + self.d * t
    }
}

pub fn reflect(v: Vec3A, n: Vec3A) -> Vec3A {
    v - 2.0 * v.dot(n) * n
}

pub fn refract(uv: Vec3A, n: Vec3A, etai_over_etat: f32) -> Vec3A {
    let cos_theta = -uv.dot(n).min(1.0);
    let r_out_perp =  etai_over_etat * (uv + cos_theta*n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    return r_out_perp + r_out_parallel;
}

// Use Schlick's approximation for reflectance.
pub fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.-ref_idx) / (1.+ref_idx);
    let r0 = r0 * r0;
    r0 + (1.-r0) * (1. - cosine).powi(5)
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl AABB {
    pub fn hit(&self, r: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for i in 0..3 {
            let inv_d = 1.0 / r.d[i];
            let mut t0 = (self.min[i] - r.o[i]) * inv_d;
            let mut t1 = (self.max[i] - r.o[i]) * inv_d;
            if inv_d < 0. {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = t_min.max(t0);
            t_max = t_max.min(t1);
            // todo: maybe "<=" to "<""
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surround(&self, rhs: Self) -> Self {
        let min = vec3a(
            self.min.x.min(rhs.min.x),
            self.min.y.min(rhs.min.y),
            self.min.z.min(rhs.min.z),
        );
        let max = vec3a(
            self.max.x.max(rhs.max.x),
            self.max.y.max(rhs.max.y),
            self.max.z.max(rhs.max.z),
        );
        Self {
            min,
            max,
        }
    }
}

pub fn smooth(v: Vec3A) -> Vec3A {
    v * v * (3. - 2. * v)
}

pub fn offset_ray(p: Vec3A, n: Vec3A) -> Vec3A {
    const ORIGIN: f32 = 1. / 32.;
    const INT_SCALE: f32 = 256.;
    const FLOAT_SCALE: f32 = 1. / 65536.;

    let of_i_x = (n.x * INT_SCALE) as i32;
    let of_i_y = (n.y * INT_SCALE) as i32;
    let of_i_z = (n.z * INT_SCALE) as i32;
    let p_i_x = unsafe {transmute::<i32, f32>(transmute::<f32, i32>(p.x) + if p.x < 0. {-of_i_x} else {of_i_x})};
    let p_i_y = unsafe {transmute::<i32, f32>(transmute::<f32, i32>(p.y) + if p.y < 0. {-of_i_y} else {of_i_y})};
    let p_i_z = unsafe {transmute::<i32, f32>(transmute::<f32, i32>(p.z) + if p.z < 0. {-of_i_z} else {of_i_z})};
    let x = if p.x.abs() < ORIGIN {p.x + n.x * FLOAT_SCALE} else {p_i_x};
    let y = if p.y.abs() < ORIGIN {p.y + n.y * FLOAT_SCALE} else {p_i_y};
    let z = if p.z.abs() < ORIGIN {p.z + n.z * FLOAT_SCALE} else {p_i_z};
    vec3a(x, y, z)
}
