//! A monotonic clock that works on every target.
//!
//! `std::time::Instant::now()` panics on `wasm32-unknown-unknown` (there is no
//! OS clock to ask), so the engine never names it directly. Import
//! [`Instant`] from here instead:
//!
//! - **native**: it *is* `std::time::Instant`, re-exported, zero cost.
//! - **wasm32**: our own type with the same API, backed by the browser's
//!   `performance.now()` (milliseconds since page load, monotonic).
//!
//! Elapsed time is always a [`std::time::Duration`] (that part of std works
//! fine on wasm), so `(Instant::now() - last).as_secs_f32()` compiles
//! unchanged on both.
//!
//! ```
//! use pulga_win_lib::time::Instant;
//!
//! let start = Instant::now();
//! let dt = Instant::now() - start;
//! assert!(dt.as_secs_f32() >= 0.0);
//! ```

pub use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use wasm::Instant;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::ops::{Add, Sub};
    use std::time::Duration;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance)]
        fn now() -> f64;
    }

    /// Point in time, stored as `performance.now()` milliseconds. Mirrors the
    /// subset of `std::time::Instant` the engine uses.
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Instant(f64);

    impl Instant {
        /// The current time.
        pub fn now() -> Self {
            Self(now())
        }

        /// Time elapsed from `earlier` to `self`; zero if `earlier` is later
        /// (same saturating behaviour as std).
        pub fn duration_since(&self, earlier: Instant) -> Duration {
            Duration::from_secs_f64(((self.0 - earlier.0) / 1000.0).max(0.0))
        }

        /// Time elapsed since `self`.
        pub fn elapsed(&self) -> Duration {
            Self::now().duration_since(*self)
        }
    }

    impl Sub for Instant {
        type Output = Duration;
        fn sub(self, earlier: Instant) -> Duration {
            self.duration_since(earlier)
        }
    }

    impl Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, d: Duration) -> Instant {
            Self(self.0 + d.as_secs_f64() * 1000.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn clock_is_monotonic() {
        let a = Instant::now();
        let b = Instant::now();
        assert!(b - a >= Duration::ZERO);
    }

    #[test]
    fn add_then_subtract_round_trips() {
        let a = Instant::now();
        let d = Duration::from_millis(250);
        let diff = (a + d) - a;
        assert!((diff.as_secs_f64() - 0.25).abs() < 1e-6);
    }
}
