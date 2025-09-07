#![allow(unsafe_op_in_unsafe_fn)]

mod extern_c;

use core_graphics::geometry::{CGPoint, CGSize};

use crate::extern_c::{
    ensure_ax_trusted, get_cgpoint, get_cgsize, get_focused_window, get_kAXPositionAttribute,
    get_kAXSizeAttribute, set_cgpoint, set_cgsize,
};

// ===== macOS AX API FFI =====
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AXError {
    Success = 0,
    // 他の値は必要になれば追加
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AXValueType {
    CGPoint = 1,
    CGSize = 2,
    // 他も必要なら
}

fn main() {
    if !ensure_ax_trusted() {
        eprintln!("Enable Accessibility permission for this app, then run again.");
        // ダイアログ後すぐはfalseになるので、許可後に再実行してください
        return;
    }

    unsafe {
        let Some(win) = get_focused_window() else {
            eprintln!("No focused window.");
            return;
        };

        let step: f64 = 10.0; // 10 pt

        if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
            let new_p = CGPoint::new(pos.x + step, pos.y);
            let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
        }

        if let Some(sz) = get_cgsize(win, get_kAXSizeAttribute()) {
            let new_s = CGSize::new((sz.width + step).max(1.0), sz.height);
            let _ = set_cgsize(win, get_kAXSizeAttribute(), new_s);
        }

        // 使った定数 CFString は解放不要（定数）
        println!("Moved right by {step}pt and widened by {step}pt.");
    }
}
