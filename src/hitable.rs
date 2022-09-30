use std::sync::Arc;

use glam::Vec3A;

use crate::math::Ray;
use crate::material::Material;

#[derive(Clone)]
pub struct HitRecord {
    pub p: Vec3A,
    pub norm: Vec3A,
    pub t: f32,
    pub front_face: bool,
    pub mat: Option<Arc<dyn Material>>,
}

impl HitRecord {
    pub fn default() -> Self {
        Self {
            p: Vec3A::ZERO,
            norm: Vec3A::ZERO,
            t: 0.0,
            front_face: true,
            mat: None,
        }
    }
    fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3A) {
        self.front_face = r.d.dot(outward_normal) < 0.0;
        self.norm = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hitable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
}

#[derive(Clone)]
pub struct Sphere {
    pub c: Vec3A,
    pub r: f32,
    pub mat: Arc<dyn Material>,
}

impl Hitable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let oc = r.o - self.c;
        let a = r.d.length_squared();
        let half_b = oc.dot(r.d);
        let c = oc.length_squared()- self.r*self.r;
        let discriminant = half_b*half_b - a*c;
        if discriminant < 0.0 {
            return false;
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.c) / self.r;
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }
}

pub type HitableList = Vec<Arc<dyn Hitable>>;

impl Hitable for HitableList {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::default();
        let mut closest_so_far = t_max;
        let mut hit_anything = false;
        for item in self.iter() {
            let be_hit = item.hit(r, t_min, closest_so_far, &mut temp_rec);
            if be_hit {
                hit_anything = true;
                closest_so_far = temp_rec.t;
            }
        }
        if hit_anything {
            *rec = temp_rec;
        }
        hit_anything
    }
}
