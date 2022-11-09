use std::f32::consts::FRAC_1_PI;
use std::sync::Arc;

use glam::*;
use rand::Rng;
use crate::math::*;
use crate::hitable::HitRecord;
use crate::texture::Texture;
use crate::lib::RNG;

pub struct ScatterRecord {
    pub spec_ray: Ray,
    pub is_spec: bool,
    pub attenuation: Vec3A,
    pub pdf: Option<Arc<dyn PDF>>,
}

impl Default for ScatterRecord {
    fn default() -> Self {
        Self { spec_ray: Default::default(), is_spec: false, attenuation: Vec3A::ONE, pdf: None }
    }
}
pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool;
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
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let uvw = ONB::build_from_w(rec.norm);
        srec.is_spec = false;
        srec.attenuation = self.albedo.value(rec.uv, rec.p);
        srec.pdf = Some(Arc::new(CosinePDF{ uvw }));
        true
    }
    fn scatter_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        rec.norm.dot(scattered.d).max(0.) * FRAC_1_PI
    }
}

pub struct Metal {
    pub albedo: Vec3A,
    pub fuzz: f32,
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let reflected = reflect(r_in.d, rec.norm) + self.fuzz * random_in_unit_sphere();
        srec.spec_ray = Ray {o: rec.p, d: reflected.normalize(), s: r_in.s};
        srec.attenuation = self.albedo;
        srec.is_spec = true;
        srec.pdf = None;
        true
    }
    fn scatter_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        rec.norm.dot(scattered.d).max(0.) * FRAC_1_PI
    }
}

pub struct Dielectric {
    pub ior: f32,
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.is_spec = true;
        srec.pdf = None;
        srec.attenuation = Vec3A::ONE;
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
        }.normalize();
        srec.spec_ray = Ray {o: rec.p, d: dir, s: r_in.s};
        true
    }
}
