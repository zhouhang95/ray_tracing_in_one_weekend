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

pub struct Lambert {
    pub albedo: Arc<dyn Texture>,
}

impl Material for Lambert {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        *scattered = Ray {o: p, d: random_on_hemisphere(rec.norm), s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p) * 2. * rec.norm.dot(scattered.d);
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
        let rnd_num = RNG.with(|rng| {
            rng.borrow_mut().gen::<f32>()
        });
        let dir = if cannot_refract || reflectance(cos_theta, ref_idx) > rnd_num {
            reflect(r_in.d, rec.norm)
        } else {
            refract(r_in.d, rec.norm, ref_idx)
        };
        *scattered = Ray {o: rec.p, d: dir, s: r_in.s};
        true
    }
}

pub struct Isotropic {
    pub albedo: Arc<dyn Texture>,
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        *scattered = Ray {
            o: rec.p,
            d: random_in_unit_sphere(),
            s: r_in.s,
        };
        *attenuation = self.albedo.value(rec.uv, rec.p);
        true
    }
}
