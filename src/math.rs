use glam::Vec3A;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub p: Vec3A,
    pub r: f32,
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