use std::sync::Arc;

use glam::*;

use rand::Rng;
use rand::seq::SliceRandom;

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

#[allow(dead_code)]
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

const PERLIN_POINT_COUNT: usize = 256;

struct Perlin {
    ranfloat: Vec<f32>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Default for Perlin {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mut ranfloat = vec![0.0f32; PERLIN_POINT_COUNT];
        for v in &mut ranfloat {
            *v = rng.gen();
        }

        let mut p = vec![0usize; PERLIN_POINT_COUNT];
        for i in 0..p.len() {
            p[i] = i;
        }
        p.shuffle(&mut rng);
        let perm_x = p.clone();
        p.shuffle(&mut rng);
        let perm_y = p.clone();
        p.shuffle(&mut rng);
        let perm_z = p.clone();

        Self {
            ranfloat,
            perm_x,
            perm_y,
            perm_z,
        }
    }
}

impl Perlin {
    pub fn noise(&self, p: Vec3A) -> f32 {
        let p = p.abs() * 4.;
        let i = p.x as usize & 255;
        let j = p.y as usize & 255;
        let k = p.z as usize & 255;
        let index = self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k];

        self.ranfloat[index]
    }
}

#[derive(Default)]
pub struct PerlinTex {
    perlin: Perlin,
}

impl Texture for PerlinTex {
    fn value(&self, _uv: Vec2, p: Vec3A) -> Vec3A {
        self.perlin.noise(p) * Vec3A::ONE
    }
}