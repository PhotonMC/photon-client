//! A value that a render thread works on while the thread that owns it goes on with other work,
//! and that waits for that work whenever it is touched again (ADR 0003, "rendering one frame
//! behind").
//!
//! The owner calls [`Handoff::submit`] with a job; the job runs on the worker with `&mut T` while
//! the owner continues. Every access through `Deref`, `DerefMut`, [`Handoff::finish`] or `Drop`
//! first waits until the job is done, so the owner never sees the value half way through a job and
//! no code outside this module needs to know that a thread exists. What the owner can do before
//! its first touch is the overlap. With no worker the job runs inline, which is the serial
//! reference.

use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

type Job<T> = Box<dyn FnOnce(&mut T) + Send>;

/// Whether a job is running, whether the last one panicked, and the first error a fallible job
/// returned that nobody has taken yet.
#[derive(Default)]
struct State {
    busy: bool,
    panicked: bool,
    error: Option<Box<dyn std::any::Any + Send>>,
    stats: HandoffStats,
}

/// What the hand-off cost, since it was made.
#[derive(Debug, Clone, Copy, Default)]
pub struct HandoffStats {
    /// Jobs the render thread (or the inline run) finished.
    pub jobs: u64,
    /// The time those jobs took.
    pub job_time: std::time::Duration,
    /// The time the owner was blocked, waiting for a job it had touched the value too soon after.
    /// It is the most that moving the first touch later could win back.
    pub waited: std::time::Duration,
}

struct Shared {
    state: Mutex<State>,
    done: Condvar,
}

/// The value's address for the worker. The worker uses it only between a `submit` and the
/// `busy` flag clearing, and the owner touches the value only after that flag clears.
struct Address<T>(NonNull<T>);

// SAFETY: see `Address`. `T: Send` is required by `Handoff::new`.
unsafe impl<T: Send> Send for Address<T> {}

struct Worker<T> {
    jobs: Sender<(Address<T>, Job<T>)>,
    thread: JoinHandle<()>,
}

pub struct Handoff<T: Send + 'static> {
    /// Made by `Box::into_raw` and freed in `Drop`. A raw pointer and not a `Box`, because the
    /// worker holds `&mut T` while the owner is waiting, and a `Box` would claim it is unique.
    value: NonNull<T>,
    shared: Arc<Shared>,
    worker: Option<Worker<T>>,
}

// SAFETY: the value moves with the handoff, and only one thread touches it at a time: the owner
// after `wait`, the worker while `busy`.
unsafe impl<T: Send + 'static> Send for Handoff<T> {}

impl<T: Send + 'static> Handoff<T> {
    /// The value, with no worker: jobs run on the caller's thread.
    pub fn serial(value: T) -> Self {
        Self::make(value, false)
    }

    /// The value, with a render thread for its jobs.
    pub fn threaded(value: T) -> Self {
        Self::make(value, true)
    }

    fn make(value: T, threaded: bool) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            done: Condvar::new(),
        });
        let worker = threaded.then(|| {
            let (jobs, queue) = channel::<(Address<T>, Job<T>)>();
            let shared = Arc::clone(&shared);
            let thread = std::thread::Builder::new()
                .name("render".into())
                .spawn(move || {
                    for (address, job) in queue {
                        // SAFETY: `Address`: the owner waits for `busy` to clear before it
                        // touches the value, so this is the only access.
                        let value = unsafe { &mut *address.0.as_ptr() };
                        let began = std::time::Instant::now();
                        let outcome =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(value)));
                        let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                        state.stats.jobs += 1;
                        state.stats.job_time += began.elapsed();
                        state.busy = false;
                        state.panicked |= outcome.is_err();
                        shared.done.notify_all();
                    }
                })
                .expect("the render thread starts");
            Worker { jobs, thread }
        });
        Self {
            value: NonNull::from(Box::leak(Box::new(value))),
            shared,
            worker,
        }
    }

    /// The value back, once the job in flight is done; the render thread ends.
    pub fn into_inner(self) -> T {
        self.wait();
        let mut this = std::mem::ManuallyDrop::new(self);
        if let Some(Worker { jobs, thread }) = this.worker.take() {
            drop(jobs);
            let _ = thread.join();
        }
        // SAFETY: `this` is never dropped, so the pointer made by `Box::leak` in `make` is read
        // back once, no job runs, and the one other owning field is dropped here.
        unsafe {
            std::ptr::drop_in_place(&raw mut this.shared);
            *Box::from_raw(this.value.as_ptr())
        }
    }

    /// Whether jobs run on a render thread.
    pub fn is_threaded(&self) -> bool {
        self.worker.is_some()
    }

    /// Runs `job` on the value: on the render thread while the owner goes on, or at once when
    /// there is none. A job still running is waited for first.
    pub fn submit(&mut self, job: impl FnOnce(&mut T) + Send + 'static) {
        self.wait();
        let Some(worker) = &self.worker else {
            let began = std::time::Instant::now();
            job(self);
            let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
            state.stats.jobs += 1;
            state.stats.job_time += began.elapsed();
            return;
        };
        self.shared
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .busy = true;
        if worker
            .jobs
            .send((Address(self.value), Box::new(job)))
            .is_err()
        {
            panic!("the render thread has stopped");
        }
    }

    /// [`submit`](Self::submit) for a job that can fail. The error does not wait for the owner:
    /// the first one is kept for [`take_error`](Self::take_error), which the owner calls once a
    /// frame.
    pub fn submit_result<E: Send + 'static>(
        &mut self,
        job: impl FnOnce(&mut T) -> Result<(), E> + Send + 'static,
    ) {
        let shared = Arc::clone(&self.shared);
        self.submit(move |value| {
            if let Err(error) = job(value) {
                let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                state.error.get_or_insert(Box::new(error));
            }
        });
    }

    /// The error a job returned, if one did and it is an `E`. Does not wait.
    pub fn take_error<E: 'static>(&self) -> Option<E> {
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let error = state.error.take()?;
        match error.downcast::<E>() {
            Ok(error) => Some(*error),
            Err(other) => {
                state.error = Some(other);
                None
            }
        }
    }

    /// Waits for the job in flight.
    ///
    /// # Panics
    ///
    /// When the job panicked: the render thread's panic is the owner's.
    pub fn finish(&self) {
        self.wait();
    }

    /// What the hand-off has cost so far. Does not wait.
    pub fn stats(&self) -> HandoffStats {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .stats
    }

    fn wait(&self) {
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.busy {
            let began = std::time::Instant::now();
            while state.busy {
                state = self
                    .shared
                    .done
                    .wait(state)
                    .unwrap_or_else(|e| e.into_inner());
            }
            state.stats.waited += began.elapsed();
        }
        assert!(!state.panicked, "a job on the render thread panicked");
    }
}

impl<T: Send + 'static> Deref for Handoff<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.wait();
        // SAFETY: no job runs after `wait`, and a new one starts only through `&mut self`.
        unsafe { self.value.as_ref() }
    }
}

impl<T: Send + 'static> DerefMut for Handoff<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.wait();
        // SAFETY: as for `deref`, and `&mut self` is the only way to the value.
        unsafe { self.value.as_mut() }
    }
}

impl<T: Send + 'static> Drop for Handoff<T> {
    fn drop(&mut self) {
        // A panic of the last job is not raised again while the owner is going away.
        {
            let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
            while state.busy {
                state = self
                    .shared
                    .done
                    .wait(state)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
        if let Some(Worker { jobs, thread }) = self.worker.take() {
            drop(jobs);
            let _ = thread.join();
        }
        // SAFETY: made by `Box::leak` in `make`, no job runs, and this is the only free.
        drop(unsafe { Box::from_raw(self.value.as_ptr()) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_serial_handoff_runs_the_job_at_once() {
        let mut value = Handoff::serial(1);
        value.submit(|n| *n += 1);
        assert_eq!(*value, 2);
        assert!(!value.is_threaded());
    }

    #[test]
    fn the_owner_goes_on_while_the_job_runs_and_waits_when_it_touches_the_value() {
        let mut value = Handoff::threaded(0_u32);
        let began = Instant::now();
        value.submit(|n| {
            std::thread::sleep(Duration::from_millis(150));
            *n = 7;
        });
        // Submitting returned long before the job ended.
        assert!(began.elapsed() < Duration::from_millis(100));
        // Reading waits for it.
        assert_eq!(*value, 7);
        assert!(began.elapsed() >= Duration::from_millis(150));
    }

    #[test]
    fn the_stats_count_jobs_and_the_time_the_owner_waited() {
        let mut value = Handoff::threaded(0);
        value.submit(|_| std::thread::sleep(Duration::from_millis(60)));
        let _ = *value;
        let stats = value.stats();
        assert_eq!(stats.jobs, 1);
        assert!(stats.job_time >= Duration::from_millis(60));
        assert!(stats.waited >= Duration::from_millis(30));
        // Touching it again with nothing in flight costs nothing.
        let _ = *value;
        assert_eq!(value.stats().waited, stats.waited);
    }

    #[test]
    fn the_value_comes_back_after_its_job() {
        let mut value = Handoff::threaded(String::from("a"));
        value.submit(|text| {
            std::thread::sleep(Duration::from_millis(30));
            text.push('b');
        });
        assert_eq!(value.into_inner(), "ab");
    }

    #[test]
    fn a_failed_job_leaves_its_first_error_for_the_owner() {
        let mut value = Handoff::threaded(0);
        value.submit_result(|_| Err::<(), _>("first"));
        value.submit_result(|_| Err::<(), _>("second"));
        value.finish();
        assert_eq!(value.take_error::<&str>(), Some("first"));
        assert_eq!(value.take_error::<&str>(), None);
    }

    #[test]
    fn jobs_run_one_after_another_in_order() {
        let mut value = Handoff::threaded(Vec::new());
        for step in 0..50 {
            value.submit(move |list: &mut Vec<i32>| list.push(step));
        }
        assert_eq!(*value, (0..50).collect::<Vec<_>>());
    }

    #[test]
    #[should_panic(expected = "panicked")]
    fn a_panic_on_the_render_thread_reaches_the_owner() {
        let mut value = Handoff::threaded(0);
        value.submit(|_| panic!("boom"));
        let _ = *value;
    }

    #[test]
    fn dropping_waits_for_the_job() {
        let seen = Arc::new(Mutex::new(false));
        {
            let mut value = Handoff::threaded(());
            let seen = Arc::clone(&seen);
            value.submit(move |()| {
                std::thread::sleep(Duration::from_millis(50));
                *seen.lock().unwrap() = true;
            });
        }
        assert!(*seen.lock().unwrap());
    }
}
