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
        let n_dot_i = rec.norm.dot(-r_in.d);
        let n_dot_o = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let h_dot_o = h.dot(dir_o);

        let fl = schlick_fresnel(n_dot_o);
        let fv = schlick_fresnel(n_dot_i);

        let fd90 = 0.5 + 2. * h_dot_o * h_dot_o * self.roughness;
        let fd = lerp(1.0, fd90, fl) * lerp(1.0, fd90, fv);

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p) * fd * 2. * n_dot_o;
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

/// Fresnel equation of a dielectric interface.
/// https://seblagarde.wordpress.com/2013/04/29/memo-on-fresnel-equations/
/// n_dot_i: abs(cos(incident angle))
/// n_dot_t: abs(cos(transmission angle))
/// eta: eta_transmission / eta_incident
pub fn fresnel_dielectric(n_dot_i: f32, n_dot_t: f32, eta: f32) -> f32 {
    assert!(n_dot_i >= 0. && n_dot_t >= 0. && eta > 0.);
    let rs = (n_dot_i - eta * n_dot_t) / (n_dot_i + eta * n_dot_t);
    let rp = (eta * n_dot_i - n_dot_t) / (eta * n_dot_i + n_dot_t);
    let f = (rs * rs + rp * rp) / 2.;
    f
}

/// https://seblagarde.wordpress.com/2013/04/29/memo-on-fresnel-equations/
/// This is a specialized version for the code above, only using the incident angle.
/// The transmission angle is derived from
/// n_dot_i: cos(incident angle) (can be negative)
/// eta: eta_transmission / eta_incident
pub fn fresnel_dielectric_2(n_dot_i: f32, eta: f32) -> f32 {
    assert!(eta > 0.);
    let n_dot_t_sq = 1. - (1. - n_dot_i * n_dot_i) / (eta * eta);
    if n_dot_t_sq < 0. {
        // total internal reflection
        return 1.;
    }
    let n_dot_t = n_dot_t_sq.sqrt();
    fresnel_dielectric(n_dot_i.abs(), n_dot_t, eta)
}

/// The masking term models the occlusion between the small mirrors of the microfacet models.
/// See Eric Heitz's paper "Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs"
/// for a great explanation.
/// https://jcgt.org/published/0003/02/03/paper.pdf
/// The derivation is based on Smith's paper "Geometrical shadowing of a random rough surface".
/// Note that different microfacet distributions have different masking terms.
pub fn smith_masking_gtr2(v_local: Vec3A, roughness: f32) -> f32{
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;
    let v2 = v_local * v_local;
    let lambda = (-1. + (1. + a2 * (v2.x + v2.y) / v2.z).sqrt()) / 2.;
    1. / (1. + lambda)
}

pub fn smith_masking_gtr2_2(v_world: Vec3A, n: Vec3A, roughness: f32) -> f32{
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;
    let v2_z_ = v_world.dot(n);
    let v2_z = v2_z_ * v2_z_;
    let lambda = (-1. + (1. + a2 * (1. - v2_z) / v2_z).sqrt()) / 2.;
    1. / (1. + lambda)
}
pub struct RoughPlastic {
    pub spec_color: Arc<dyn Texture>,
    pub diff_color: Arc<dyn Texture>,
    pub roughness: f32,
    pub eta: f32,
}

impl Material for RoughPlastic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_i = rec.norm.dot(-r_in.d);
        let n_dot_o = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let h_dot_i = h.dot(-r_in.d);
        let h_dot_o = h.dot(dir_o);
        let n_dot_h = rec.norm.dot(h);


        let kd = self.diff_color.value(rec.uv, rec.p);
        let ks = self.spec_color.value(rec.uv, rec.p);

        let roughness = self.roughness.clamp(0.01, 1.);
        let f_o = fresnel_dielectric_2(h_dot_o, self.eta);
        let d = gtr2(n_dot_h, roughness);
        let g = smith_masking_gtr2_2(-r_in.d, rec.norm, roughness)
                    *smith_masking_gtr2_2(dir_o, rec.norm, roughness);
        let spec_contrib = ks * (g * f_o * d) / (4. * n_dot_i * n_dot_o);

        let f_i = fresnel_dielectric_2(h_dot_i, self.eta);
        let diff_contrib = kd * (1. - f_o) * (1. - f_i) * FRAC_1_PI;

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = (spec_contrib + diff_contrib) * n_dot_o * 2. * PI;
        true
    }
}


pub struct DisneyDiffuse {
    pub albedo: Arc<dyn Texture>,
    pub roughness: f32,
    pub subsurface: f32,
}

impl Material for DisneyDiffuse {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_i = rec.norm.dot(-r_in.d);
        let n_dot_o = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let h_dot_o = h.dot(dir_o);

        let fo = schlick_fresnel(n_dot_o);
        let fi = schlick_fresnel(n_dot_i);

        let fd90 = 0.5 + 2. * h_dot_o * h_dot_o * self.roughness;
        let fd = lerp(1.0, fd90, fo) * lerp(1.0, fd90, fi);

        let fss90 = self.roughness * h_dot_o * h_dot_o;
        let fss_wi = lerp(1., fss90, fi);
        let fss_wo = lerp(1., fss90, fo);
        let fss = 1.25 * (fss_wi * fss_wo * (1. / (n_dot_i + n_dot_o) - 0.5) + 0.5);


        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = self.albedo.value(rec.uv, rec.p) * lerp(fd, fss, self.subsurface) * 2. * n_dot_o;
        true
    }
}
pub struct DisneyMetal {
    pub albedo: Arc<dyn Texture>,
    pub roughness: f32,
    pub anisotropic: f32,
}


impl Material for DisneyMetal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_i = rec.norm.dot(-r_in.d);
        let n_dot_o = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let h_dot_o = h.dot(dir_o);

        let albedo = self.albedo.value(rec.uv, rec.p);


        let fm = Vec3A::ONE.lerp(albedo, schlick_fresnel(h_dot_o));

        let aspect = (1. - 0.9 * self.anisotropic).sqrt();
        let alpha_min = 0.0001;
        let alpha_x = (self.roughness * self.roughness / aspect).max(alpha_min);
        let alpha_y = (self.roughness * self.roughness * aspect).max(alpha_min);

        let h_local = Vec3A::ONE;

        // let factor =
        // let dm = 1. / (alpha_x * alpha_y * ().powi(2)) * FRAC_1_PI;


        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        true
    }
}


pub struct DisneySheen {
    pub albedo: Arc<dyn Texture>,
    pub tint: f32,
}

impl Material for DisneySheen {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let p = offset_hit_point(rec.p, rec.norm);
        let dir_o = random_on_hemisphere(rec.norm);
        let n_dot_i = rec.norm.dot(-r_in.d);
        let n_dot_o = rec.norm.dot(dir_o);
        let h = (dir_o - r_in.d).normalize();
        let h_dot_i = h.dot(-r_in.d);
        let h_dot_o = h.dot(dir_o);
        let n_dot_h = rec.norm.dot(h);

        let albedo = self.albedo.value(rec.uv, rec.p);

        let luminance = vec3a(0.3, 0.6, 0.1).dot(albedo);
        let c_tint = if luminance > 0. { albedo / luminance } else { Vec3A::ONE };
        let c_sheen = Vec3A::ONE.lerp(c_tint, self.tint);
        let f_sheen = c_sheen * schlick_fresnel(h_dot_o);

        *scattered = Ray {o: p, d: dir_o, s: r_in.s};
        *attenuation = f_sheen * n_dot_o * 2. * PI;
        true
    }
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
