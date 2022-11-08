use std::default;
use std::f32::consts::PI;
use std::sync::Arc;

use glam::*;
use image::flat::NormalForm;
use rand::Rng;

use crate::math::*;
use crate::material::Material;
use crate::lib::RNG;
use crate::texture::Texture;

#[derive(Default, Clone)]
pub struct HitRecord {
    pub p: Vec3A,
    pub norm: Vec3A,
    pub tang: Vec3A,
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
    pub fn world_to_local(&self, v: Vec3A) -> Vec3A {
        let bitang = self.norm.cross(self.tang);
        vec3a(v.dot(self.tang), v.dot(bitang), v.dot(self.norm))
    }
    pub fn world_to_local_with_rot(&self, v: Vec3A, rot: f32) -> Vec3A {
        let tang = rot.cos() * self.tang - rot.sin() * self.norm.cross(self.tang);
        let bitang = self.norm.cross(tang);
        vec3a(v.dot(tang), v.dot(bitang), v.dot(self.norm))
    }
}

impl std::fmt::Display for HitRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rec: p: {}, n: {}, t: {}, front: {}, uv: {}", self.p, self.norm, self.t, self.front_face, self.uv)
    }
}

pub trait Hitable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
    fn bbox(&self, aabb: &mut AABB) -> bool;
    fn memo(&self) -> String;
    fn pdf_value(&self, o: Vec3A, v: Vec3A) -> f32 {
        0.
    }
    fn random(&self, o: Vec3A) -> Vec3A {
        Vec3A::X
    }
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
        rec.tang = Vec3A::Y.cross(outward_normal).normalize();
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
        if p.x < self.min.x || p.x > self.max.x || p.y < self.min.y || p.y > self.max.y {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.xy();
        rec.p = p;
        rec.t = t;

        let outward_normal = Vec3A::Z;
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x, self.min.y, self.min.z - 0.0001);
        aabb.max = vec3a(self.max.x, self.max.y, self.max.z + 0.0001);
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
        if p.x < self.min.x || p.x > self.max.x || p.z < self.min.z || p.z > self.max.z {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.xz();
        rec.p = p;
        rec.t = t;

        let outward_normal = Vec3A::Y;
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x, self.min.y - 0.0001, self.min.z);
        aabb.max = vec3a(self.max.x, self.max.y + 0.0001, self.max.z);
        true
    }
    fn memo(&self) -> String {
        "XZRect".into()
    }
    fn pdf_value(&self, o: Vec3A, v: Vec3A) -> f32 {
        let mut rec = HitRecord::default();
        let dir = v.normalize();
        let r = &Ray { o, d: dir, s: Vec2::ZERO };
        if !self.hit(r, 0.001, f32::INFINITY, &mut rec) {
            return 0.;
        }
        let xyz_len = self.max - self.min;
        let area = xyz_len.x * xyz_len.z;
        let distance_squared = rec.t * rec.t;
        let cosine = dir.dot(rec.norm);

        distance_squared / (cosine * area)
    }
    fn random(&self, o: Vec3A) -> Vec3A {
        vec3a(
            random_range(self.min.x, self.max.x),
            self.min.y,
            random_range(self.min.z, self.max.z),
        ) - o
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
        if p.z < self.min.z || p.z > self.max.z || p.y < self.min.y || p.y > self.max.y {
            return false;
        }

        let uv = (p - self.min) / (self.max - self.min);
        rec.uv = uv.yz();
        rec.p = p;
        rec.t = t;

        let outward_normal = Vec3A::X;
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        aabb.min = vec3a(self.min.x - 0.0001, self.min.y, self.min.z);
        aabb.max = vec3a(self.max.x + 0.0001, self.max.y, self.max.z);
        true
    }
    fn memo(&self) -> String {
        "YZRect".into()
    }
}

pub struct GBox {
    pub aabb: AABB,
    pub sides: HitableList,
}

impl GBox {
    pub fn new(min: Vec3A, max: Vec3A, mat: Arc<dyn Material>) -> Self {
        let sides: HitableList = vec![
            Arc::new(XYRect { min: vec3a(min.x, min.y, min.z), max: vec3a(max.x, max.y, min.z), mat: mat.clone() }),
            Arc::new(XYRect { min: vec3a(min.x, min.y, max.z), max: vec3a(max.x, max.y, max.z), mat: mat.clone() }),
            Arc::new(XZRect { min: vec3a(min.x, min.y, min.z), max: vec3a(max.x, min.y, max.z), mat: mat.clone() }),
            Arc::new(XZRect { min: vec3a(min.x, max.y, min.z), max: vec3a(max.x, max.y, max.z), mat: mat.clone() }),
            Arc::new(YZRect { min: vec3a(min.x, min.y, min.z), max: vec3a(min.x, max.y, max.z), mat: mat.clone() }),
            Arc::new(YZRect { min: vec3a(max.x, min.y, min.z), max: vec3a(max.x, max.y, max.z), mat: mat.clone() }),
        ];
        let aabb = AABB { min, max };

        Self { aabb, sides }
    }
}

impl Hitable for GBox {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let res = self.sides.hit(r, t_min, t_max, rec);
        // if res {
        //     eprintln!("{}", rec);
        // }
        res
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        *aabb = self.aabb;
        true
    }

    fn memo(&self) -> String {
        todo!()
    }
}

pub struct Translate {
    pub offset: Vec3A,
    pub ptr: Arc<dyn Hitable>,
}

impl Hitable for Translate {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let moved_r = Ray { o: r.o - self.offset, d: r.d, s: r.s};
        if self.ptr.hit(&moved_r, t_min, t_max, rec) {
            rec.p += self.offset;
            true
        } else {
            false
        }
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        let mut get_aabb = AABB::default();
        if self.ptr.bbox(&mut get_aabb) {
            *aabb = AABB {
                min: get_aabb.min + self.offset,
                max: get_aabb.max + self.offset,
            };
            true
        } else {
            false
        }
    }

    fn memo(&self) -> String {
        todo!()
    }
}

pub struct RotateY {
    ptr: Arc<dyn Hitable>,
    angle: f32,
    sin_theta: f32,
    cos_theta: f32,
    has_box: bool,
    aabb: AABB,
}

impl RotateY {
    pub fn new(ptr: Arc<dyn Hitable>, angle: f32) -> Self {
        let radians = angle.to_radians();
        let (sin_theta, cos_theta) = radians.sin_cos();
        let mut aabb_ = AABB::default();
        let has_box = ptr.bbox(&mut aabb_);
        let mut min = vec3a(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = if i == 0 {aabb_.min.x} else {aabb_.max.x};
                    let y = if j == 0 {aabb_.min.y} else {aabb_.max.y};
                    let z = if k == 0 {aabb_.min.z} else {aabb_.max.z};

                    let new_x = cos_theta * x + sin_theta * z;
                    let new_z = -sin_theta * x + cos_theta * z;

                    let tester = vec3a(new_x, y, new_z);

                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }
        let aabb = AABB { min, max };

        Self { ptr, angle, sin_theta, cos_theta, has_box, aabb }
    }
}


impl Hitable for RotateY {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let mut o = r.o;
        let mut d = r.d;

        o.x = self.cos_theta * r.o.x - self.sin_theta * r.o.z;
        o.z = self.sin_theta * r.o.x + self.cos_theta * r.o.z;

        d.x = self.cos_theta * r.d.x - self.sin_theta * r.d.z;
        d.z = self.sin_theta * r.d.x + self.cos_theta * r.d.z;

        let rot_r = Ray {o, d, s: r.s};

        if self.ptr.hit(&rot_r, t_min, t_max, rec) {
            let mut p = rec.p;
            let mut n = rec.norm;

            p.x =  self.cos_theta * rec.p.x + self.sin_theta * rec.p.z;
            p.z = -self.sin_theta * rec.p.x + self.cos_theta * rec.p.z;

            n.x =  self.cos_theta * rec.norm.x + self.sin_theta * rec.norm.z;
            n.z = -self.sin_theta * rec.norm.x + self.cos_theta * rec.norm.z;

            rec.p = p;
            rec.set_face_normal(&rot_r, n);
            true
        } else {
            false
        }
    }

    fn bbox(&self, aabb: &mut AABB) -> bool {
        *aabb = self.aabb;
        self.has_box
    }

    fn memo(&self) -> String {
        todo!()
    }
}

