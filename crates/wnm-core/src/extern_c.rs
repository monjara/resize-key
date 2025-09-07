#![allow(unsafe_op_in_unsafe_fn)]

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

unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;

    pub fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    pub fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    pub fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    pub fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;

    pub fn AXValueCreate(value_type: AXValueType, value_ptr: *const c_void) -> CFTypeRef;
    pub fn AXValueGetType(value: CFTypeRef) -> AXValueType;
    pub fn AXValueGetValue(
        value: CFTypeRef,
        value_type: AXValueType,
        value_ptr: *mut c_void,
    ) -> Boolean;

    // Functions to get AX constants from C
    pub fn get_kAXFocusedApplicationAttribute() -> CFStringRef;
    pub fn get_kAXFocusedWindowAttribute() -> CFStringRef;
    pub fn get_kAXPositionAttribute() -> CFStringRef;
    pub fn get_kAXSizeAttribute() -> CFStringRef;
    fn get_kAXTrustedCheckOptionPrompt() -> CFStringRef;

    // Additional functions for getting frontmost app
    fn GetFrontProcess(psn: *mut ProcessSerialNumber) -> i32;
    fn GetProcessPID(psn: *const ProcessSerialNumber, pid: *mut i32) -> i32;
}

// ===== RAII Wrapper for AXValue =====
struct OwnedAxValue(NonNull<c_void>);

impl OwnedAxValue {
    unsafe fn new<T>(value_type: AXValueType, value: &T) -> Option<Self> {
        let v = AXValueCreate(value_type, (value as *const T).cast());
        NonNull::new(v as *mut c_void).map(Self)
    }

    unsafe fn from_copy(value: CFTypeRef) -> Option<Self> {
        NonNull::new(value as *mut c_void).map(Self)
    }

    fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr()
    }
}

impl Drop for OwnedAxValue {
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
pub enum __AXUIElement {}
pub type AXUIElementRef = *const __AXUIElement;

pub unsafe fn cfstring_ref(s: CFStringRef) -> CFString {
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

pub unsafe fn get_focused_window() -> Option<AXUIElementRef> {
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

trait AxKind {
    type Pod; // 実データ型（CGPoint/CGSize）
    const TYPE: AXValueType; // 対応する AXValueType
}

struct AsCGPoint;
impl AxKind for AsCGPoint {
    type Pod = CGPoint;
    const TYPE: AXValueType = AXValueType::CGPoint;
}

struct AsCGSize;
impl AxKind for AsCGSize {
    type Pod = CGSize;
    const TYPE: AXValueType = AXValueType::CGSize;
}

fn ax_bool(b: Boolean) -> bool {
    b != 0
}

unsafe fn get_ax<K: AxKind<Pod: Default>>(
    elem: AXUIElementRef,
    key: CFStringRef,
) -> Option<K::Pod> {
    let raw = copy_attr(elem, key)?;
    let owned = OwnedAxValue::from_copy(raw)?;

    if AXValueGetType(owned.as_ptr()) != K::TYPE {
        return None;
    }

    let mut out = K::Pod::default();
    if ax_bool(AXValueGetValue(
        owned.as_ptr(),
        K::TYPE,
        (&mut out as *mut K::Pod).cast(),
    )) {
        Some(out)
    } else {
        None
    }
}

pub unsafe fn get_cgpoint(elem: AXUIElementRef, key: CFStringRef) -> Option<CGPoint> {
    get_ax::<AsCGPoint>(elem, key)
}

pub unsafe fn get_cgsize(elem: AXUIElementRef, key: CFStringRef) -> Option<CGSize> {
    get_ax::<AsCGSize>(elem, key)
}

unsafe fn set_ax<T: Copy>(
    value_type: AXValueType,
    elem: AXUIElementRef,
    key: CFStringRef,
    value: T,
) -> anyhow::Result<()> {
    let ax_value = OwnedAxValue::new(value_type, &value)
        .ok_or_else(|| anyhow::anyhow!("Failed to create AXValue"))?;

    let err = AXUIElementSetAttributeValue(elem, key, ax_value.as_ptr());
    if err == AXError::Success {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to set ax attribute"))
    }
    // ax_value は自動的にドロップされ、リソースが解放される
}

pub unsafe fn set_cgpoint(elem: AXUIElementRef, key: CFStringRef, p: CGPoint) -> bool {
    set_ax(AXValueType::CGPoint, elem, key, p).is_ok()
}

pub unsafe fn set_cgsize(elem: AXUIElementRef, key: CFStringRef, s: CGSize) -> bool {
    set_ax(AXValueType::CGSize, elem, key, s).is_ok()
}

pub fn ensure_ax_trusted() -> bool {
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
