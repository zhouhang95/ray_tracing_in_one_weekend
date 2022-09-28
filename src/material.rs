use glam::Vec3A;
use crate::math::*;
use crate::hitable::HitRecord;

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool;
}

pub struct Lambertian {
    pub albedo: Vec3A,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool {
        let mut scatter_direction = rec.norm + random_in_unit_sphere().normalize();
        if vec3a_near_zero(scatter_direction) {
            scatter_direction = rec.norm;
        }
        *scattered = Ray {o: rec.p, d: scatter_direction.normalize()};
        *attenuation = self.albedo;
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
        *scattered = Ray {o: rec.p, d: reflected.normalize()};
        *attenuation = self.albedo;
        reflected.dot(rec.norm) > 0.
    }
}
