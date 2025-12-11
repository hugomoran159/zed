use crate::{PlatformDispatcher, RunnableVariant, TaskLabel, TaskTiming, ThreadTaskTimings};
use parking_lot::Mutex;
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::Arc,
    time::Duration,
};
use wasm_bindgen::prelude::*;
use web_time::Instant;

thread_local! {
    static MAIN_THREAD_MARKER: RefCell<bool> = const { RefCell::new(false) };
}

fn is_main_thread() -> bool {
    MAIN_THREAD_MARKER.with(|marker| *marker.borrow())
}

fn mark_as_main_thread() {
    MAIN_THREAD_MARKER.with(|marker| *marker.borrow_mut() = true);
}

struct WebDispatcherState {
    main_thread_runnables: VecDeque<RunnableVariant>,
}

pub(crate) struct WebDispatcher {
    state: Arc<Mutex<WebDispatcherState>>,
}

impl WebDispatcher {
    pub fn new() -> Self {
        mark_as_main_thread();

        Self {
            state: Arc::new(Mutex::new(WebDispatcherState {
                main_thread_runnables: VecDeque::new(),
            })),
        }
    }

    pub fn run_on_main_thread(&self) {
        loop {
            let runnable = {
                let mut state = self.state.lock();
                state.main_thread_runnables.pop_front()
            };

            match runnable {
                Some(RunnableVariant::Meta(runnable)) => { runnable.run(); },
                Some(RunnableVariant::Compat(runnable)) => { runnable.run(); },
                None => break,
            }
        }
    }
}

impl PlatformDispatcher for WebDispatcher {
    fn is_main_thread(&self) -> bool {
        is_main_thread()
    }

    fn dispatch(&self, runnable: RunnableVariant, _label: Option<TaskLabel>) {
        wasm_bindgen_futures::spawn_local(async move {
            match runnable {
                RunnableVariant::Meta(runnable) => { runnable.run(); },
                RunnableVariant::Compat(runnable) => { runnable.run(); },
            }
        });
    }

    fn dispatch_on_main_thread(&self, runnable: RunnableVariant) {
        self.state.lock().main_thread_runnables.push_back(runnable);

        let state = self.state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let runnable = {
                    let mut state_guard = state.lock();
                    state_guard.main_thread_runnables.pop_front()
                };

                match runnable {
                    Some(RunnableVariant::Meta(runnable)) => { runnable.run(); },
                    Some(RunnableVariant::Compat(runnable)) => { runnable.run(); },
                    None => break,
                }
            }
        });
    }

    fn dispatch_after(&self, duration: Duration, runnable: RunnableVariant) {
        let millis = duration.as_millis() as i32;

        let closure = Closure::once(move || {
            match runnable {
                RunnableVariant::Meta(runnable) => { runnable.run(); },
                RunnableVariant::Compat(runnable) => { runnable.run(); },
            }
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
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn get_all_timings(&self) -> Vec<ThreadTaskTimings> {
        Vec::new()
    }

    fn get_current_thread_timings(&self) -> Vec<TaskTiming> {
        Vec::new()
    }
}
