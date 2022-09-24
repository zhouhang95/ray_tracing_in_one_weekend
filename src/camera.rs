use glam::Vec3A;
use glam::vec3a;

use crate::math::Ray;

pub struct Camera {
    origin: Vec3A,
    horizontal: Vec3A,
    vertical: Vec3A,
    lower_left_corner: Vec3A,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;
        let origin = vec3a(0.0, 0.0, 0.0);
        let horizontal = vec3a(viewport_width, 0.0, 0.0);
        let vertical = vec3a(0.0, viewport_height, 0.0);
        let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - vec3a(0.0, 0.0, focal_length);
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
        }
    }
}