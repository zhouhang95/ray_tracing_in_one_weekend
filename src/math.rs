use glam::{Vec3A, vec3a};
use rand::Rng;

pub fn vec3a_random() -> Vec3A {
    let mut rng = rand::thread_rng();
    vec3a(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>())
}

pub fn random_in_unit_sphere() -> Vec3A {
    loop {
        let v = vec3a_random() * 2.0 - Vec3A::ONE;
        if v.length_squared() < 1.0 {
            return v;
        }
    }
}

pub fn random_in_hemisphere(norm: Vec3A) -> Vec3A {
    let v = random_in_unit_sphere();
    if v.dot(norm) > 0.0 {
        v
    } else {
        -v
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub p: Vec3A,
    pub norm: Vec3A,
    pub t: f32,
    pub front_face: bool,
}

impl HitRecord {
    pub fn default() -> Self {
        Self {
            p: Vec3A::ZERO,
            norm: Vec3A::ZERO,
            t: 0.0,
            front_face: true,
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

pub trait Hitable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub c: Vec3A,
    pub r: f32,
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

        true
    }
}

pub type HitableList = Vec<Box<dyn Hitable>>;

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
                *rec = temp_rec;
            }
        }
        hit_anything
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Vec3A,
    pub d: Vec3A,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.o + self.d * t
    }
}