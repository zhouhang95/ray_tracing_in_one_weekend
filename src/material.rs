use std::f32::consts::FRAC_1_PI;
use std::sync::Arc;

use glam::*;
use rand::Rng;
use crate::math::*;
use crate::hitable::HitRecord;
use crate::texture::Texture;
use crate::lib::RNG;

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray, pdf: &mut f32) -> bool;
    fn scatter_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        0.
    }
    fn emitted(&self, _uv: Vec2, _p: Vec3A) -> Vec3A {
        Vec3A::ZERO
    }
}

pub struct Emission {
    pub emit: Arc<dyn Texture>,
}

impl Material for Emission {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _attenuation: &mut Vec3A, _scattered: &mut Ray, _pdf: &mut f32) -> bool {
        false
    }

    fn emitted(&self, uv: Vec2, p: Vec3A) -> Vec3A {
        self.emit.value(uv, p)
    }
}

pub struct Diffuse {
    pub albedo: Arc<dyn Texture>,
}

impl Material for Diffuse {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray, pdf: &mut f32) -> bool {
        let uvw = ONB::build_from_w(rec.norm);
        let mut scatter_direction = uvw.local(random_cosine_dir());
        if vec3a_near_zero(scatter_direction) {
            scatter_direction = rec.norm;
        }
        let p = offset_hit_point(rec.p, rec.norm);
        *scattered = Ray {o: p, d: scatter_direction.normalize(), s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p);
        *pdf = rec.norm.dot(scattered.d) * FRAC_1_PI;
        true
    }
    fn scatter_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        rec.norm.dot(scattered.d).max(0.) * FRAC_1_PI
    }
}
