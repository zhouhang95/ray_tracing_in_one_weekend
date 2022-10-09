use std::cell::RefCell;
use rand::SeedableRng;
use rand::rngs::SmallRng;

thread_local! {
    pub static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::seed_from_u64(1995));
}
