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

fn trilinear_interp(c: &[[[f32;2];2];2], uvw: Vec3A) -> f32 {
    let mut accum = 0.;
    let u = uvw.x;
    let v = uvw.y;
    let w = uvw.z;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                accum += c[i][j][k]
                    * if i == 1 {u} else {1.-u}
                    * if j == 1 {v} else {1.-v}
                    * if k == 1 {w} else {1.-w};
            }
        }
    }
    accum
}

impl Perlin {
    pub fn noise(&self, p: Vec3A) -> f32 {
        let i = p.x.floor() as isize;
        let j = p.y.floor() as isize;
        let k = p.z.floor() as isize;

        let mut c = [[[0f32;2];2];2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let index =
                        self.perm_x[((i + di).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize] ^
                        self.perm_y[((j + dj).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize] ^
                        self.perm_z[((k + dk).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize];
                    
                    c[di as usize][dj as usize][dk as usize] = self.ranfloat[index];
                }
            }
        }
        let uvw = p - p.floor();
        let uvw = crate::math::smooth(uvw);
        trilinear_interp(&c, uvw)
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