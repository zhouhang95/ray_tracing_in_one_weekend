use glam::Vec3A;

pub struct Ray {
    pub org: Vec3A,
    pub dir: Vec3A,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.org + self.dir * t
    }
}