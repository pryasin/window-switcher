use std::mem::size_of;

use windows::core::PCSTR;
use windows::Win32::{
    Foundation::{BOOL, HWND},
    Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND},
    System::LibraryLoader::{GetProcAddress, LoadLibraryA},
};

#[repr(C)]
struct AccentPolicy {
    accent_state: u32,
    accent_flags: u32,
    gradient_color: u32,
    animation_id: u32,
}

#[repr(C)]
struct WindowCompositionAttributeData {
    attribute: u32,
    data: *mut core::ffi::c_void,
    size_of_data: usize,
}

const WCA_ACCENT_POLICY: u32 = 19;
const ACCENT_ENABLE_BLURBEHIND: u32 = 3;
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

type SetWindowCompositionAttributeFn =
    unsafe extern "system" fn(hwnd: HWND, data: *mut WindowCompositionAttributeData) -> BOOL;

pub fn apply_overlay_blur(hwnd: HWND) -> bool {
    unsafe { try_apply_accent(hwnd) || try_apply_dwm_blur(hwnd) }
}

unsafe fn try_apply_accent(hwnd: HWND) -> bool {
    let module = match LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr())) {
        Ok(module) => module,
        Err(_) => return false,
    };

    let proc = match GetProcAddress(module, PCSTR(b"SetWindowCompositionAttribute\0".as_ptr())) {
        Some(proc) => proc,
        None => return false,
    };

    let set_window_composition_attribute: SetWindowCompositionAttributeFn =
        core::mem::transmute(proc);

    let mut accent = AccentPolicy {
        accent_state: ACCENT_ENABLE_ACRYLICBLURBEHIND,
        accent_flags: 0,
        // ARGB: alpha must be non-zero for acrylic to activate
        gradient_color: 0x7F1E1E1E,
        animation_id: 0,
    };

    let mut data = WindowCompositionAttributeData {
        attribute: WCA_ACCENT_POLICY,
        data: &mut accent as *mut _ as *mut core::ffi::c_void,
        size_of_data: size_of::<AccentPolicy>(),
    };

    if set_window_composition_attribute(hwnd, &mut data).as_bool() {
        return true;
    }

    accent.accent_state = ACCENT_ENABLE_BLURBEHIND;
    set_window_composition_attribute(hwnd, &mut data).as_bool()
}

unsafe fn try_apply_dwm_blur(hwnd: HWND) -> bool {
    let bb = DWM_BLURBEHIND {
        dwFlags: DWM_BB_ENABLE,
        fEnable: true.into(),
        ..Default::default()
    };

    DwmEnableBlurBehindWindow(hwnd, &bb).is_ok()
}
