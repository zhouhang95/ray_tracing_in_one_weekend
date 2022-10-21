use std::f32::consts::*;
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
        let dir_o = random_on_hemisphere(rec.norm);
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
        *attenuation = self.albedo.value(rec.uv, rec.p) * w * 2. * cos_o;
        true
    }
}


pub struct BurleyDiffuse {
    pub albedo: Arc<dyn Texture>,
    pub roughness: f32,
}

impl Material for BurleyDiffuse {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_l = rec.norm.dot(-r_in.d);
        let n_dot_v = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let l_dot_h = h.dot(-r_in.d);

        let fl = schlick_fresnel(n_dot_l);
        let fv = schlick_fresnel(n_dot_v);

        let fd90 = 0.5 + 2. * l_dot_h * l_dot_h * self.roughness;
        let fd = lerp(1.0, fd90, fl) * lerp(1.0, fd90, fv);

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p) * fd;
        true
    }
}

// normal distribution function
fn gtr1(n_dot_h: f32, a: f32) -> f32 {
    if a >= 1. {
        return FRAC_1_PI;
    }
    let a2 = a * a;
    let t = 1. + (a2 - 1.) * n_dot_h * n_dot_h;
    (a2 - 1.) / (PI * a2.ln() * t)
}

fn gtr2(n_dot_h: f32, a: f32) -> f32 {
    let a2 = a * a;
    let t = 1. + (a2 - 1.) * n_dot_h * n_dot_h;
    a2 / (PI * t * t)
}

// geometric distribution function
fn smith_geo_ggx(n_dot_v: f32, alpha: f32) -> f32 {
    let a = alpha * alpha;
    let b = n_dot_v * n_dot_v;
    1. / (n_dot_v + (a + b - a * b).sqrt())
}


pub struct Clearcoat {
    pub clearcoat_gloss: f32,
}

impl Material for Clearcoat {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_l = rec.norm.dot(-r_in.d);
        let n_dot_v = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let l_dot_h = h.dot(-r_in.d);
        let n_dot_h = rec.norm.dot(h);

        let fh = schlick_fresnel(l_dot_h);

        let dr = gtr1(n_dot_h, lerp(0.1, 0.001, self.clearcoat_gloss));
        let fr = lerp(0.04, 1.0, fh);
        let gr = smith_geo_ggx(n_dot_l, 0.25) * smith_geo_ggx(n_dot_v, 0.25);
        let cc = 0.25 * gr * fr * dr;

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = Vec3A::splat(cc);
        true
    }
}
