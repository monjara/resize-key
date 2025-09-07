#![allow(non_snake_case)]

use objc2_app_kit::{NSApp, NSApplicationActivationPolicy, NSStatusBar};
use objc2_foundation::MainThreadMarker;
use wnm_core::hotkey::{move_by, resize_from_left, resize_from_right};

// Carbon HotKey
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn RegisterEventHotKey(
        key: u32,
        mods: u32,
        id: EventHotKeyID,
        target: *mut core::ffi::c_void,
        opts: u32,
        out: *mut *mut core::ffi::c_void,
    ) -> i32;
    fn GetEventDispatcherTarget() -> *mut core::ffi::c_void;
    fn InstallEventHandler(
        t: *mut core::ffi::c_void,
        cb: unsafe extern "C" fn(
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
        ) -> i32,
        count: u32,
        types: *const (u32, u32),
        user: *mut core::ffi::c_void,
        out: *mut *mut core::ffi::c_void,
    ) -> i32;
    fn GetEventParameter(
        e: *mut core::ffi::c_void,
        name: u32,
        ty: u32,
        out_ty: *mut u32,
        in_sz: u32,
        out_sz: *mut u32,
        out: *mut core::ffi::c_void,
    ) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

const EVENT_CLASS_KEYBOARD: u32 = 0x6B657962;
const EVENT_HOTKEY_PRESSED: u32 = 0;
const EVENT_PARAM_NAME: u32 = 0x6B686964;
const TYPE_EVENT_HOTKEY_ID: u32 = 0x6B686964;
const CMD_KEY: u32 = 1 << 8;
const SHIFT_KEY: u32 = 1 << 9;
const OPTION_KEY: u32 = 1 << 11;
const KC_L: u32 = 0x7B;
const KC_R: u32 = 0x7C;
const KC_D: u32 = 0x7D;
const KC_U: u32 = 0x7E;

static mut STEP: f64 = 10.0;

unsafe extern "C" fn handler(
    _c: *mut core::ffi::c_void,
    e: *mut core::ffi::c_void,
    _u: *mut core::ffi::c_void,
) -> i32 {
    let mut id = EventHotKeyID {
        signature: 0,
        id: 0,
    };
    let mut out: u32 = 0;
    let _ = unsafe {
        GetEventParameter(
            e,
            EVENT_PARAM_NAME,
            TYPE_EVENT_HOTKEY_ID,
            std::ptr::null_mut(),
            std::mem::size_of::<EventHotKeyID>() as u32,
            &mut out,
            (&mut id) as *mut _ as _,
        )
    };
    let s = unsafe { STEP };
    match id.id {
        0 => {
            let _ = move_by(s, 0.0);
        } // ⌥⌘→
        1 => {
            let _ = move_by(-s, 0.0);
        } // ⌥⌘←
        2 => {
            let _ = move_by(0.0, -s);
        } // ⌥⌘↑  （y軸は環境で反転なら符号調整）
        3 => {
            let _ = move_by(0.0, s);
        } // ⌥⌘↓
        4 => {
            let _ = resize_from_right(s);
        } // ⌥⌘⇧→ 右辺を外へ=広がる
        5 => {
            let _ = resize_from_right(-s);
        } // ⌥⌘⇧← 右辺を内へ=狭まる
        6 => {
            let _ = resize_from_left(s);
        } // ⌥⌘⇧↑ 左辺を外へ=広がる
        7 => {
            let _ = resize_from_left(-s);
        } // ⌥⌘⇧↓ 左辺を内へ=狭まる
        _ => {}
    }
    0
}

unsafe fn reg(idx: u32, key: u32, mods: u32) {
    let id = EventHotKeyID {
        signature: 0x41584B59,
        id: idx,
    };

    let mut out = std::ptr::null_mut();
    let _ = unsafe { RegisterEventHotKey(key, mods, id, GetEventDispatcherTarget(), 0, &mut out) };
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();

    unsafe {
        let app = NSApp(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        // ステータスバー（最小限の実装）
        let _sb = NSStatusBar::systemStatusBar();
        // Note: objc2のAPIが複雑なため、UI作成は省略してホットキーのみ実装

        // HotKey登録
        let spec = (EVENT_CLASS_KEYBOARD, EVENT_HOTKEY_PRESSED);
        let mut _h: *mut core::ffi::c_void = std::ptr::null_mut();
        InstallEventHandler(
            GetEventDispatcherTarget(),
            handler,
            1,
            &spec,
            std::ptr::null_mut(),
            &mut _h,
        );

        let m = OPTION_KEY | CMD_KEY;
        let r = OPTION_KEY | CMD_KEY | SHIFT_KEY;
        reg(0, KC_R, m);
        reg(1, KC_L, m);
        reg(2, KC_U, m);
        reg(3, KC_D, m);
        reg(4, KC_R, r);
        reg(5, KC_L, r);
        reg(6, KC_U, r);
        reg(7, KC_D, r);

        // アプリケーション実行
        app.run();
    }
}
