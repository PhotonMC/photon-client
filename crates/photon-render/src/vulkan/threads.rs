//! Thread priorities, the one place the process asks the kernel for them.

/// Run the calling thread below the frame's: a busy pool of meshers must
/// not take a core from the main thread or the driver's threads, which
/// shows as frames running long. It changes only when the work gets done.
pub fn yield_to_the_frame() {
    #[cfg(target_os = "linux")]
    {
        unsafe extern "C" {
            fn setpriority(which: i32, who: u32, priority: i32) -> i32;
        }
        // SAFETY: `setpriority` takes plain integers; on Linux a `who` of 0
        // names the calling thread. A failure leaves the priority as it was.
        unsafe {
            setpriority(0, 0, 10);
        }
    }
}
