use std::{ffi::c_void, ptr::NonNull};

use core_foundation::{
    base::{CFRelease, TCFType},
    boolean::CFBoolean,
    string::CFString,
};
use core_graphics::{
    display::CFDictionary,
    geometry::{CGPoint, CGSize},
};

// ===== RAII Wrapper for AXValue =====
struct AXValueWrapper(NonNull<c_void>);

impl AXValueWrapper {
    unsafe fn new_cgpoint(point: CGPoint) -> Option<Self> {
        let v = AXValueCreate(AXValueType::CGPoint, &point as *const _ as *const c_void);
        NonNull::new(v as *mut c_void).map(Self)
    }

    unsafe fn new_cgsize(size: CGSize) -> Option<Self> {
        let v = AXValueCreate(AXValueType::CGSize, &size as *const _ as *const c_void);
        NonNull::new(v as *mut c_void).map(Self)
    }

    fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr()
    }
}

impl Drop for AXValueWrapper {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0.as_ptr() as _);
        }
    }
}

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

type Boolean = u8; // macOSのBooleanはUInt8
type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type CFDictionaryRef = *const c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ProcessSerialNumber {
    high_long_of_psn: u32,
    low_long_of_psn: u32,
}

#[allow(non_camel_case_types)]
pub(crate) enum __AXUIElement {}
pub(crate) type AXUIElementRef = *const __AXUIElement;

unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;

    pub(crate) fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    pub(crate) fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    pub(crate) fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    pub(crate) fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;

    pub(crate) fn AXValueCreate(value_type: AXValueType, value_ptr: *const c_void) -> CFTypeRef;
    pub(crate) fn AXValueGetType(value: CFTypeRef) -> AXValueType;
    pub(crate) fn AXValueGetValue(
        value: CFTypeRef,
        value_type: AXValueType,
        value_ptr: *mut c_void,
    ) -> Boolean;

    // Functions to get AX constants from C
    pub(crate) fn get_kAXFocusedApplicationAttribute() -> CFStringRef;
    pub(crate) fn get_kAXFocusedWindowAttribute() -> CFStringRef;
    pub(crate) fn get_kAXPositionAttribute() -> CFStringRef;
    pub(crate) fn get_kAXSizeAttribute() -> CFStringRef;
    fn get_kAXTrustedCheckOptionPrompt() -> CFStringRef;

    // Additional functions for getting frontmost app
    fn GetFrontProcess(psn: *mut ProcessSerialNumber) -> i32;
    fn GetProcessPID(psn: *const ProcessSerialNumber, pid: *mut i32) -> i32;
}

// ===== ユーティリティ =====
pub(crate) unsafe fn cfstring_ref(s: CFStringRef) -> CFString {
    // 定数 CFStringRef を安全にラップ
    CFString::wrap_under_get_rule(s as *const _)
}

unsafe fn copy_attr(element: AXUIElementRef, key: CFStringRef) -> Option<CFTypeRef> {
    let mut out: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(element, key, &mut out);
    if err == AXError::Success && !out.is_null() {
        Some(out)
    } else {
        None
    }
}

unsafe fn get_frontmost_app_pid() -> Option<i32> {
    let mut psn = ProcessSerialNumber {
        high_long_of_psn: 0,
        low_long_of_psn: 0,
    };
    let err = GetFrontProcess(&mut psn);

    if err != 0 {
        return None;
    }

    let mut pid: i32 = 0;
    let err = GetProcessPID(&psn, &mut pid);

    if err != 0 {
        return None;
    }

    Some(pid)
}

pub(crate) unsafe fn get_focused_window() -> Option<AXUIElementRef> {
    // Try method 1: Get focused app from system-wide element
    let sys = AXUIElementCreateSystemWide();

    let app = match copy_attr(sys, get_kAXFocusedApplicationAttribute()) {
        Some(val) => val as AXUIElementRef,
        None => {
            // Try method 2: Get frontmost application by PID
            let pid = get_frontmost_app_pid()?;
            AXUIElementCreateApplication(pid)
        }
    };

    let win_val = copy_attr(app, get_kAXFocusedWindowAttribute())?;
    Some(win_val as AXUIElementRef)
}

pub(crate) unsafe fn get_cgpoint(elem: AXUIElementRef, key: CFStringRef) -> Option<CGPoint> {
    struct Guard(CFTypeRef);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe {
                CFRelease(self.0 as _);
            }
        }
    }

    let v = copy_attr(elem, key)?;
    let _guard = Guard(v);

    let vt = AXValueGetType(v);
    if vt != AXValueType::CGPoint {
        return None;
    }

    let mut p = CGPoint::new(0.0, 0.0);
    let ok = AXValueGetValue(v, AXValueType::CGPoint, &mut p as *mut _ as *mut c_void);
    if ok != 0 { Some(p) } else { None }
}

pub(crate) unsafe fn get_cgsize(elem: AXUIElementRef, key: CFStringRef) -> Option<CGSize> {
    struct Guard(CFTypeRef);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe {
                CFRelease(self.0 as _);
            }
        }
    }

    let v = copy_attr(elem, key)?;
    let _guard = Guard(v);

    let vt = AXValueGetType(v);
    if vt != AXValueType::CGSize {
        return None;
    }

    let mut s = CGSize::new(0.0, 0.0);
    let ok = AXValueGetValue(v, AXValueType::CGSize, &mut s as *mut _ as *mut c_void);
    if ok != 0 { Some(s) } else { None }
}

pub(crate) unsafe fn set_cgpoint(
    elem: AXUIElementRef,
    key: CFStringRef,
    p: CGPoint,
) -> anyhow::Result<()> {
    let ax_value = AXValueWrapper::new_cgpoint(p)
        .ok_or_else(|| anyhow::anyhow!("Failed to create AXValue for CGPoint"))?;

    let err = AXUIElementSetAttributeValue(elem, key, ax_value.as_ptr());
    if err == AXError::Success {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to set CGPoint attribute"))
    }
    // ax_value は自動的にドロップされ、リソースが解放される
}

pub(crate) unsafe fn set_cgsize(elem: AXUIElementRef, key: CFStringRef, s: CGSize) -> bool {
    let ax_value = match AXValueWrapper::new_cgsize(s) {
        Some(value) => value,
        None => return false,
    };

    let err = AXUIElementSetAttributeValue(elem, key, ax_value.as_ptr());
    err == AXError::Success
    // ax_value は自動的にドロップされ、リソースが解放される
}

pub(crate) fn ensure_ax_trusted() -> bool {
    unsafe {
        // 許可ダイアログを出す
        let key = cfstring_ref(get_kAXTrustedCheckOptionPrompt());
        let dict = CFDictionary::from_CFType_pairs(&[(
            key.as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as CFDictionaryRef) != 0
    }
}
