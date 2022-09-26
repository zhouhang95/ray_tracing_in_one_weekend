use glam::{Vec3A, vec3a};
use rand::Rng;

pub fn vec3a_near_zero(v: Vec3A) -> bool {
    let s = 1e-4;
    return (v.x.abs() < s) && (v.y.abs() < s) && (v.z.abs() < s);
}

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
pub struct Ray {
    pub o: Vec3A,
    pub d: Vec3A,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.o + self.d * t
    }
}