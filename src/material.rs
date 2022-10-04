use std::sync::Arc;

use glam::Vec3A;
use rand::Rng;
use crate::math::*;
use crate::hitable::HitRecord;
use crate::texture::Texture;

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool;
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let mut scatter_direction = rec.norm + random_in_unit_sphere().normalize();
        if vec3a_near_zero(scatter_direction) {
            scatter_direction = rec.norm;
        }
        *scattered = Ray {o: rec.p, d: scatter_direction.normalize(), s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p);
        true
    }
}

pub struct Metal {
    pub albedo: Vec3A,
    pub fuzz: f32,
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let reflected = reflect(r_in.d, rec.norm) + self.fuzz * random_in_unit_sphere();
        *scattered = Ray {o: rec.p, d: reflected.normalize(), s: r_in.s};
        *attenuation = self.albedo;
        reflected.dot(rec.norm) > 0.
    }
}

pub struct Dielectric {
    pub ior: f32,
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        *attenuation = Vec3A::ONE;
        let ref_idx = if rec.front_face { 1.0 / self.ior } else { self.ior };
        let cos_theta = -r_in.d.dot(rec.norm).min(1.0);
        let sin_thera = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract =  sin_thera * ref_idx > 1.;
        let mut rng = rand::thread_rng();
        let dir = if cannot_refract || reflectance(cos_theta, ref_idx) > rng.gen::<f32>() {
            reflect(r_in.d, rec.norm)
        } else {
            refract(r_in.d, rec.norm, ref_idx)
        };
        *scattered = Ray {o: rec.p, d: dir, s: r_in.s};
        true
    }
}