use std::mem::transmute;
use std::f32::consts::{PI, FRAC_1_PI};

use glam::*;
use rand::Rng;

use crate::lib::RNG;

pub fn vec3a_near_zero(v: Vec3A) -> bool {
    let s = f32::EPSILON;
    return (v.x.abs() < s) && (v.y.abs() < s) && (v.z.abs() < s);
}

pub fn vec3a_near_one(v: Vec3A) -> bool {
    (v.length() - 1.).abs() < 1e-6
}

pub fn vec2_random() -> Vec2 {
    RNG.with(|rng| {
        let x = rng.borrow_mut().gen::<f32>();
        let y = rng.borrow_mut().gen::<f32>();
        vec2(x, y)
    })
}

pub fn vec3a_random() -> Vec3A {
    RNG.with(|rng| {
        let x = rng.borrow_mut().gen::<f32>();
        let y = rng.borrow_mut().gen::<f32>();
        let z = rng.borrow_mut().gen::<f32>();
        vec3a(x, y, z)
    })
}

pub fn random_range(min: f32, max: f32) -> f32 {
    RNG.with(|rng| {
        rng.borrow_mut().gen_range(min..max)
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
pub fn random_on_unit_sphere() -> Vec3A {
    random_in_unit_sphere().normalize()
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
pub fn random_on_hemisphere(norm: Vec3A) -> Vec3A {
    random_in_hemisphere(norm).normalize()
}

pub fn random_cosine_dir() -> Vec3A {
    let v = vec2_random();
    let r1 = v[0];
    let r2 = v[1];

    let r2_sqrt = r2.sqrt();

    let z = (1. - r2).sqrt();

    let phi = 2. * PI * r1;
    let x = phi.cos() * r2_sqrt;
    let y = phi.sin() * r2_sqrt;

    vec3a(x, y, z).normalize()
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

pub fn schlick_fresnel(u: f32) -> f32 {
    (1. - u).powi(5)
}

// Use Schlick's approximation for reflectance.
pub fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.-ref_idx) / (1.+ref_idx);
    let r0 = r0 * r0;
    r0 + (1.-r0) * schlick_fresnel(cosine)
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

pub fn offset_hit_point(p: Vec3A, n: Vec3A) -> Vec3A {
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

pub fn lerp(from: f32, to: f32, s: f32) -> f32 {
    from + (to - from) * s
}

pub struct ONB {
    pub u: Vec3A,
    pub v: Vec3A,
    pub w: Vec3A,
}


impl ONB {
    pub fn local(&self, v: Vec3A) -> Vec3A {
        v.x * self.u + v.y * self.v + v.z * self.w
    }

    pub fn build_from_w(n: Vec3A) -> Self {
        let w = n;
        let a = if w.x.abs() > 0.9 {
            Vec3A::Y
        } else {
            Vec3A::X
        };
        let v = w.cross(a);
        let u = v.cross(w);
        Self {
            u, v, w,
        }
    }
}

pub trait PDF {
    fn value(&self, dir: Vec3A) -> f32;
    fn gen(&self) -> Vec3A;
}

pub struct CosinePDF {
    pub uvw: ONB,
}

impl CosinePDF {
    pub fn new(w: Vec3A) -> Self {
        Self {
            uvw: ONB::build_from_w(w)
        }
    }
}

impl PDF for CosinePDF {
    fn value(&self, dir: Vec3A) -> f32 {
        dir.dot(self.uvw.w).max(0.) * FRAC_1_PI
    }

    fn gen(&self) -> Vec3A {
        self.uvw.local(random_cosine_dir()).normalize()
    }
}