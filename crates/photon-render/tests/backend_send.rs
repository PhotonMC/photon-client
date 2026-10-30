//! The render thread of #22 owns the backend, so the backend has to move between threads.

fn assert_send<T: Send>() {}

#[test]
fn the_backend_can_move_to_a_render_thread() {
    assert_send::<photon_render::VulkanBackend>();
}
