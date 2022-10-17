use std::cell::RefCell;
use glam::Vec3A;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use once_cell::sync::OnceCell;

thread_local! {
    pub static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::seed_from_u64(1995));
}

pub static SKY_COLOR: OnceCell<fn(Vec3A) -> Vec3A> = OnceCell::new();
