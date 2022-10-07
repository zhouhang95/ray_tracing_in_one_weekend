use std::f32::consts::PI;
use std::sync::Arc;

use glam::*;
use rand::Rng;

use crate::math::*;
use crate::material::Material;
use crate::lib::RNG;

#[derive(Default, Clone)]
pub struct HitRecord {
    pub p: Vec3A,
    pub norm: Vec3A,
    pub t: f32,
    pub front_face: bool,
    pub mat: Option<Arc<dyn Material>>,
    pub uv: Vec2,
}

impl HitRecord {
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
    fn bbox(&self, aabb: &mut AABB) -> bool;
    fn memo(&self) -> String;
}

#[derive(Clone)]
pub struct Sphere {
    pub c: Vec3A,
    pub r: f32,
    pub mat: Arc<dyn Material>,
    pub name: String,
}

impl Sphere {
    pub fn get_uv(n: Vec3A) -> Vec2 {
        let theta = (-n.y).acos();
        let phi = (-n.z).atan2(n.x) + PI;
        let u = phi / (2. * PI);
        let v = theta / PI;
        vec2(u, v)
    }
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
        rec.uv = Sphere::get_uv(outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = self.c - self.r;
        aabb.max = self.c + self.r;
        true
    }
    fn memo(&self) -> String {
        self.name.clone()
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
    fn bbox(&self, aabb: &mut AABB) -> bool {
        if self.is_empty() {
            return false;
        }
        let mut bbox = AABB::default();
        let mut first = true;
        for o in self {
            if o.bbox(&mut bbox) {
                if first {
                    *aabb = bbox;
                    first = false;
                } else {
                    *aabb = aabb.surround(bbox);
                }
            } else {
                return false;
            }
        }
        true
    }
    fn memo(&self) -> String {
        "HitableList".to_string()
    }
}

pub struct BvhNode {
    aabb: AABB,
    left: Arc<dyn Hitable>,
    right: Arc<dyn Hitable>,
}

fn box_compare(a: &Arc<dyn Hitable>, b: &Arc<dyn Hitable>, axis: usize) -> std::cmp::Ordering {
    let mut box_a = AABB::default();
    let mut box_b = AABB::default();

    let a_result = a.bbox(&mut box_a);
    let b_result = b.bbox(&mut box_b);
    if a_result == false && b_result == false {
        eprintln!("No bounding box in bvh_node constructor.");
    }
    box_a.min[axis].total_cmp(&box_b.min[axis])
}

impl BvhNode {
    pub fn new(
        objects: &mut HitableList,
        start: usize,
        end: usize,
    ) -> Self {
        let axis: usize = RNG.with(|rng| {
            rng.borrow_mut().gen_range(0..3)
        });
        let object_span = end - start;
        let (left, right) = match object_span {
            0 => unimplemented!(),
            1 => (objects[start].clone(), objects[start].clone()),
            v => {
                objects[start..end].sort_by(|a, b| box_compare(a, b, axis));
                if v == 2 {
                    (objects[start].clone(), objects[start+1].clone())
                } else {
                    let mid = start + v / 2;
                    let left: Arc<dyn Hitable> = Arc::new(BvhNode::new(
                        objects,
                        start,
                        mid,
                    ));
                    let right: Arc<dyn Hitable> = Arc::new(BvhNode::new(
                        objects,
                        mid,
                        end,
                    ));
                    (left, right)
                }
            },
        };

        let mut box_a = AABB::default();
        let mut box_b = AABB::default();

        let a_result = left.bbox(&mut box_a);
        let b_result = right.bbox(&mut box_b);
        if a_result == false && b_result == false {
            eprintln!("No bounding box in bvh_node constructor.");
        }
        let aabb = box_a.surround(box_b);

        Self { aabb, left, right }
    }
}

impl Hitable for BvhNode {
    fn memo(&self) -> String {
        "BvhNode".to_string()
    }
    fn bbox(&self, aabb: &mut AABB) -> bool {
        *aabb = self.aabb;
        true
    }
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        if self.aabb.hit(r, t_min, t_max) == false {
            return false;
        }
        let hit_left = self.left.hit(r, t_min, t_max, rec);
        let hit_right = self.right.hit(r, t_min, if hit_left {rec.t} else {t_max}, rec);
        // eprintln!("{}, {}: {}, {}",(r.s[0] * 400.) as i32,  (r.s[1] * 200.) as i32, hit_left, hit_right);
        hit_left || hit_right
    }
}


#[derive(Clone)]
pub struct XYRect {
    pub min: Vec3A,
    pub max: Vec3A,
    pub mat: Arc<dyn Material>,
}

impl Hitable for XYRect {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let t = (self.min.z - r.o.z) / r.d.z;
        if t < t_min || t > t_max {
            return false;
        }
        let p = r.at(t);
        if p.x < self.min.x || p.x > self.max.x || p.y < self.min.y || p.y > self.min.y {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.xy();
        rec.p = p;
        rec.t = t;

        let outward_normal = vec3a(0., 0., 1.);
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x, self.min.y, self.min.z - 0.0001);
        aabb.max = vec3a(self.max.x, self.max.y, self.min.z + 0.0001);
        true
    }
    fn memo(&self) -> String {
        "XYRect".into()
    }
}

#[derive(Clone)]
pub struct XZRect {
    pub min: Vec3A,
    pub max: Vec3A,
    pub mat: Arc<dyn Material>,
}

impl Hitable for XZRect {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let t = (self.min.y - r.o.y) / r.d.y;
        if t < t_min || t > t_max {
            return false;
        }
        let p = r.at(t);
        if p.x < self.min.x || p.x > self.max.x || p.z < self.min.z || p.z > self.min.z {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.xz();
        rec.p = p;
        rec.t = t;

        let outward_normal = vec3a(0., 1., 0.);
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x, self.min.y - 0.0001, self.min.z);
        aabb.max = vec3a(self.max.x, self.max.y + 0.0001, self.min.z);
        true
    }
    fn memo(&self) -> String {
        "XZRect".into()
    }
}

#[derive(Clone)]
pub struct YZRect {
    pub min: Vec3A,
    pub max: Vec3A,
    pub mat: Arc<dyn Material>,
}

impl Hitable for YZRect {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let t = (self.min.x - r.o.x) / r.d.x;
        if t < t_min || t > t_max {
            return false;
        }
        let p = r.at(t);
        if p.z < self.min.z || p.z > self.max.z || p.y < self.min.y || p.y > self.min.y {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.yz();
        rec.p = p;
        rec.t = t;

        let outward_normal = vec3a(1., 0., 0.);
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x - 0.0001, self.min.y, self.min.z);
        aabb.max = vec3a(self.max.x + 0.0001, self.max.y, self.min.z);
        true
    }
    fn memo(&self) -> String {
        "YZRect".into()
    }
}
