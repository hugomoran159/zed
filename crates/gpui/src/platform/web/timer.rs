use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};
use wasm_bindgen::prelude::*;
use web_time::Instant;

/// A WASM-compatible timer that resolves after a given duration.
/// This is a drop-in replacement for `smol::Timer` on WASM targets.
pub struct Timer {
    duration: Duration,
    started: bool,
}

impl Timer {
    /// Create a new timer that will resolve after the given duration.
    pub fn after(duration: Duration) -> Self {
        Self {
            duration,
            started: false,
        }
    }
}

impl Future for Timer {
    type Output = Instant;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            let waker = cx.waker().clone();
            let millis = self.duration.as_millis() as i32;

            let closure = Closure::once(move || {
                waker.wake();
            });

            if let Some(window) = web_sys::window() {
                if window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        millis,
                    )
                    .is_ok()
                {
                    closure.forget();
                }
            }
            Poll::Pending
        } else {
            Poll::Ready(Instant::now())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::after(Duration::from_millis(100));
        assert!(!timer.started);
        assert_eq!(timer.duration, Duration::from_millis(100));
    }

    #[test]
    fn test_timer_zero_duration() {
        let timer = Timer::after(Duration::ZERO);
        assert_eq!(timer.duration, Duration::ZERO);
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        async fn test_timer_resolves() {
            let start = Instant::now();
            let timer = Timer::after(Duration::from_millis(50));
            let _instant = timer.await;
            let elapsed = start.elapsed();
            assert!(elapsed >= Duration::from_millis(40));
        }

        #[wasm_bindgen_test]
        async fn test_timer_zero_duration_resolves_quickly() {
            let start = Instant::now();
            let timer = Timer::after(Duration::ZERO);
            let _instant = timer.await;
            let elapsed = start.elapsed();
            assert!(elapsed < Duration::from_millis(50));
        }

        #[wasm_bindgen_test]
        async fn test_multiple_timers() {
            let start = Instant::now();

            let timer1 = Timer::after(Duration::from_millis(20));
            let timer2 = Timer::after(Duration::from_millis(40));

            timer1.await;
            let elapsed1 = start.elapsed();
            assert!(elapsed1 >= Duration::from_millis(15));

            timer2.await;
            let elapsed2 = start.elapsed();
            assert!(elapsed2 >= Duration::from_millis(35));
        }
    }
}
