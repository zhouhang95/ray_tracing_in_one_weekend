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
