use std::sync::Arc;

use glam::*;
use rand::Rng;
use crate::math::*;
use crate::hitable::HitRecord;
use crate::texture::Texture;
use crate::lib::RNG;

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool;
    fn emitted(&self, _uv: Vec2, _p: Vec3A) -> Vec3A {
        Vec3A::ZERO
    }
}

pub struct Emission {
    pub emit: Arc<dyn Texture>,
}

impl Material for Emission {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _attenuation: &mut Vec3A, _scattered: &mut Ray) -> bool {
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let mut scatter_direction = rec.norm + random_in_unit_sphere().normalize();
        if vec3a_near_zero(scatter_direction) {
            scatter_direction = rec.norm;
        }
        let p = offset_hit_point(rec.p, rec.norm);
        *scattered = Ray {o: p, d: scatter_direction.normalize(), s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p);
        true
    }
}
