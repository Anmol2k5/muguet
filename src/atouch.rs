// muguet - atouch.rs (Android only)
// Minimal touch boundary: forwards winit touch events into the shared input
// block that the game already reads. Semantics are deliberately menu-grade:
// the primary contact acts as the left mouse button at an absolute position.
// Full gesture mapping (sticks, buttons, camera) is M8 work; this boundary
// exists so touches do something safe and predictable instead of nothing.
//
// Multi-touch: only the first contact is tracked; extra fingers are ignored.

use std::sync::atomic::{AtomicU64, Ordering};

/// Finger id + 1 of the tracked contact; 0 = none.
static PRIMARY: AtomicU64 = AtomicU64::new(0);

pub fn touch(window: &winit::window::Window, t: &winit::event::Touch) {
    use winit::event::TouchPhase::*;
    let finger = t.id + 1;
    match t.phase {
        Started => {
            if PRIMARY.compare_exchange(0, finger, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                pos(window, t.location.x, t.location.y);
                crate::input::button(0, true);
            }
        }
        Moved => {
            if PRIMARY.load(Ordering::SeqCst) == finger {
                pos(window, t.location.x, t.location.y);
            }
        }
        Ended | Cancelled => {
            if PRIMARY.compare_exchange(finger, 0, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                crate::input::button(0, false);
            }
        }
    }
}

/// Same physical-to-backbuffer mapping as App::cursor.
fn pos(window: &winit::window::Window, x: f64, y: f64) {
    let s = window.inner_size();
    let (w, h) = crate::input::size();
    crate::input::mouse_pos(
        (x * w as f64 / s.width.max(1) as f64).round() as i32,
        (y * h as f64 / s.height.max(1) as f64).round() as i32,
    );
}
