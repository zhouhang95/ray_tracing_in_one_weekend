use glam::{Vec3A, vec3a};
use crate::{math::Ray, hitable::HitRecord};

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Vec3A, scattered: &mut Ray) -> bool;
}

