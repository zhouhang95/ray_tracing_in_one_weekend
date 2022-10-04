use std::sync::Arc;

use glam::*;

pub trait Texture: Send + Sync {
    fn value(&self, uv: Vec2, p: Vec3A) -> Vec3A;
}

pub struct ConstantTex {
    pub col: Vec3A,
}

impl Texture for ConstantTex {
    fn value(&self, _uv: Vec2, _p: Vec3A) -> Vec3A {
        self.col
    }
}

pub struct CheckerTex {
    odd: Arc<dyn Texture>,
    even: Arc<dyn Texture>,
}
impl CheckerTex {
    pub fn new(odd_col: Vec3A, even_col: Vec3A) -> Self {
        Self {
            odd: Arc::new(ConstantTex {col: odd_col}),
            even: Arc::new(ConstantTex {col: even_col}),
        }
    }
}

impl Texture for CheckerTex {
    fn value(&self, uv: Vec2, p: Vec3A) -> Vec3A {
        let sines = (p.x * 10.).sin() * (p.y * 10.).sin() * (p.z * 10.).sin();
        if sines < 0. {
            self.odd.value(uv, p)
        } else {
            self.even.value(uv, p)
        }
    }
}