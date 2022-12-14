use std::sync::Arc;

use glam::*;

use image::*;
use rand::seq::SliceRandom;

use crate::math::vec3a_random_range;
use crate::lib::RNG;

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
    rand_vec: Vec<Vec3A>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Default for Perlin {
    fn default() -> Self {
        let mut rand_vec = vec![Vec3A::ZERO; PERLIN_POINT_COUNT];
        for v in &mut rand_vec {
            *v = vec3a_random_range(-1., 1.);
        }

        let mut p = vec![0usize; PERLIN_POINT_COUNT];
        for i in 0..p.len() {
            p[i] = i;
        }
        RNG.with(|rng| {
            p.shuffle(&mut *rng.borrow_mut());
        });
        let perm_x = p.clone();
        RNG.with(|rng| {
            p.shuffle(&mut *rng.borrow_mut());
        });
        let perm_y = p.clone();
        RNG.with(|rng| {
            p.shuffle(&mut *rng.borrow_mut());
        });
        let perm_z = p.clone();

        Self {
            rand_vec,
            perm_x,
            perm_y,
            perm_z,
        }
    }
}

fn trilinear_interp(c: &[[[Vec3A;2];2];2], uvw: Vec3A) -> f32 {
    let mut accum = 0.;
    let uvw2 = crate::math::smooth(uvw);

    let u = uvw2.x;
    let v = uvw2.y;
    let w = uvw2.z;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight = uvw - vec3a(i as f32, j as f32, k as f32);
                accum += c[i][j][k].dot(weight)
                    * if i == 1 {u} else {1.-u}
                    * if j == 1 {v} else {1.-v}
                    * if k == 1 {w} else {1.-w};
            }
        }
    }
    accum
}

impl Perlin {
    pub fn turb(&self, mut p: Vec3A) -> f32 {
        let mut accum = 0.;
        let mut w = 1.;
        for _ in 0..7 {
            accum += w * self.noise(p);
            p *= 2.;
            w *= 0.5;
        }
        accum.abs()
    }
    pub fn noise(&self, p: Vec3A) -> f32 {
        let i = p.x.floor() as isize;
        let j = p.y.floor() as isize;
        let k = p.z.floor() as isize;

        let mut c = [[[Vec3A::ZERO;2];2];2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let index =
                        self.perm_x[((i + di).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize] ^
                        self.perm_y[((j + dj).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize] ^
                        self.perm_z[((k + dk).rem_euclid(PERLIN_POINT_COUNT as isize)) as usize];
                    
                    c[di as usize][dj as usize][dk as usize] = self.rand_vec[index];
                }
            }
        }
        let uvw = p - p.floor();
        trilinear_interp(&c, uvw)
    }
}

pub struct PerlinTex {
    perlin: Perlin,
    scale: f32,
}


impl PerlinTex {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            perlin: Perlin::default(),
        }
    }
}

impl Texture for PerlinTex {
    fn value(&self, _uv: Vec2, p: Vec3A) -> Vec3A {
        ((10. * self.perlin.turb(p) + self.scale * p.z).sin() + 1.) * 0.5 * Vec3A::ONE
    }
}
#[derive(Debug)]
pub struct ImageTex {
    img: ImageBuffer<Rgb<f32>, Vec<f32>>,
}

impl ImageTex {
    pub fn new(path: String) -> Self {
        let img = image::open(path).unwrap();
        let img = img.to_rgb32f();
        Self {
            img,
        }
    }
}

impl Texture for ImageTex {
    fn value(&self, uv: Vec2, _p: Vec3A) -> Vec3A {
        let u = uv.x.clamp(0., 1.);
        let v = 1. - uv.y.clamp(0., 1.);
        let i = ((u * self.img.width() as f32) as u32).clamp(0, self.img.width()-1);
        let j = ((v * self.img.height() as f32) as u32).clamp(0, self.img.height()-1);
        let rgb = self.img.get_pixel(i, j);
        vec3a(rgb[0], rgb[1], rgb[2])
    }
}