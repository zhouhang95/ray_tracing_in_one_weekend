use glam::*;

use crate::math::Ray;

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Vec3A,
    horizontal: Vec3A,
    vertical: Vec3A,
    lower_left_corner: Vec3A,
}

impl Camera {
    pub fn new(
        lookfrom: Vec3A,
        lookat: Vec3A,
        vup: Vec3A,
        vfov: f32,
        aspect_ratio: f32,
    ) -> Self {
        let origin = lookfrom;
        let theta = vfov.to_radians();
        let viewport_height = (theta / 2.0).tan() * 2.;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = origin - horizontal / 2. - vertical / 2. - w;
        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        }
    }
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray{
            o: self.origin,
            d: (self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin).normalize(),
            s: vec2(u, v),
        }
    }
}