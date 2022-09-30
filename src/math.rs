use glam::{Vec3A, vec3a};
use rand::Rng;

pub fn vec3a_near_zero(v: Vec3A) -> bool {
    let s = f32::EPSILON;
    return (v.x.abs() < s) && (v.y.abs() < s) && (v.z.abs() < s);
}

pub fn vec3a_random() -> Vec3A {
    let mut rng = rand::thread_rng();
    vec3a(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>())
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