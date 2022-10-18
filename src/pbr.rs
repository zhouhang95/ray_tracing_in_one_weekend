use std::sync::Arc;

use glam::*;

use crate::hitable::HitRecord;
use crate::material::Material;
use crate::math::*;
use crate::texture::Texture;


pub struct OrenNayar {
    pub albedo: Arc<dyn Texture>,
    pub roughness: f32,
}

impl Material for OrenNayar {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_in_hemisphere(rec.norm);
        let cos_i = rec.norm.dot(r_in.d).abs();
        let cos_o = rec.norm.dot(dir_o);
        let sin_i = (1. - cos_i * cos_i).sqrt();
        let sin_o = (1. - cos_o * cos_o).sqrt();
        let max_cos = (cos_i * cos_o + sin_i * sin_o).max(0.);

        let r2 = self.roughness * self.roughness;
        let a = 1. - 0.5 * r2 / (r2 + 0.33);
        let b = 0.45 * r2 / (r2 + 0.09);

        let (sin_alpha, tan_beta) = if cos_i > cos_o {
            (sin_o, sin_i/cos_i)
        } else {
            (sin_i, sin_o/cos_o)
        };
        let w = a + b * max_cos * sin_alpha * tan_beta;

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p) * w;
        true
    }
}