use std::sync::Mutex;
use image::RgbaImage;
use log::info;

use crate::ui::overlay::{CaptureOutcome, OverlaySession};

#[cfg(windows)]
static SESSION_PTR: Mutex<Option<Box<OverlaySession>>> = Mutex::new(None);
#[cfg(windows)]
static OUTCOME: Mutex<Option<CaptureOutcome>> = Mutex::new(None);
#[cfg(windows)]
static PINNED_IMAGE: Mutex<Option<RgbaImage>> = Mutex::new(None);

#[cfg(windows)]
pub fn run_overlay_window(screenshot: RgbaImage) -> Result<CaptureOutcome, String> {
    use windows::core::w;
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::Graphics::Gdi::HBRUSH;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;

    #[cfg(windows)]
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }

    let session = Box::new(OverlaySession::new(screenshot));
    *SESSION_PTR.lock().unwrap() = Some(session);

    let final_outcome = unsafe {
        let h_instance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW error: {e}"))?;
        let h_instance_typed = HINSTANCE(h_instance.0);
        let class_name = w!("HyperShotOverlayWindow");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(overlay_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance_typed,
            hIcon: crate::ui::daemon::load_tray_icon(),
            hCursor: LoadCursorW(None, IDC_CROSS).unwrap_or_default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: crate::ui::daemon::load_tray_icon(),
        };

        let _ = RegisterClassExW(&wnd_class);

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let width = if vw > 0 { vw } else { GetSystemMetrics(SM_CXSCREEN) };
        let height = if vh > 0 { vh } else { GetSystemMetrics(SM_CYSCREEN) };

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("HyperShot"),
            WS_POPUP | WS_VISIBLE,
            vx,
            vy,
            width,
            height,
            None,
            None,
            Some(h_instance_typed),
            None,
        ).map_err(|e| format!("Failed to create overlay window: {e}"))?;

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        OUTCOME.lock().unwrap().take().unwrap_or(CaptureOutcome::Cancelled)
    };

    Ok(final_outcome)
}

#[cfg(windows)]
unsafe fn check_and_handle_pending_save(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, ShowWindow, SW_HIDE, SW_SHOW};

    let needs_save = {
        let mut session_guard = SESSION_PTR.lock().unwrap();
        if let Some(session) = session_guard.as_mut() {
            if session.pending_save {
                session.pending_save = false;
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if needs_save {
        let _ = ReleaseCapture();
        let _ = ShowWindow(hwnd, SW_HIDE);

        let saved = {
            let mut session_guard = SESSION_PTR.lock().unwrap();
            if let Some(session) = session_guard.as_mut() {
                session.perform_save_dialog()
            } else {
                false
            }
        };

        if saved {
            let outcome = {
                let session_guard = SESSION_PTR.lock().unwrap();
                session_guard.as_ref().map(|s| s.outcome.clone()).unwrap_or(CaptureOutcome::Cancelled)
            };
            *OUTCOME.lock().unwrap() = Some(outcome);
            let _ = DestroyWindow(hwnd);
            return true;
        } else {
            // User cancelled save dialog: restore overlay!
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
    false
}

#[cfg(windows)]
unsafe extern "system" fn overlay_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyState, ReleaseCapture, SetCapture, VK_CONTROL, VK_ESCAPE, VK_RETURN,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;

    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut session_guard = SESSION_PTR.lock().unwrap();
            if let Some(session) = session_guard.as_mut() {
                let (w, h) = (session.width, session.height);
                let pixels = session.render_frame();

                let bi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: w as i32,
                        biHeight: -(h as i32), // Top-down
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let _ = SetDIBitsToDevice(
                    hdc,
                    0,
                    0,
                    w,
                    h,
                    0,
                    0,
                    0,
                    h,
                    pixels.as_ptr() as *const _,
                    &bi,
                    DIB_RGB_COLORS,
                );
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut session_guard = SESSION_PTR.lock().unwrap();
            if let Some(session) = session_guard.as_mut() {
                session.on_mouse_move(x, y);
            }

            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            SetCapture(hwnd);

            let mut finished = false;
            {
                let mut session_guard = SESSION_PTR.lock().unwrap();
                if let Some(session) = session_guard.as_mut() {
                    session.on_mouse_down(x, y);
                    if session.is_finished {
                        finished = true;
                        *OUTCOME.lock().unwrap() = Some(session.outcome.clone());
                    }
                }
            }

            if finished {
                let _ = ReleaseCapture();
                let _ = DestroyWindow(hwnd);
                return LRESULT(0);
            }

            if check_and_handle_pending_save(hwnd) {
                return LRESULT(0);
            }

            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = ReleaseCapture();

            let mut finished = false;
            {
                let mut session_guard = SESSION_PTR.lock().unwrap();
                if let Some(session) = session_guard.as_mut() {
                    session.on_mouse_up(x, y);
                    if session.is_finished {
                        finished = true;
                        *OUTCOME.lock().unwrap() = Some(session.outcome.clone());
                    }
                }
            }

            if finished {
                let _ = DestroyWindow(hwnd);
                return LRESULT(0);
            }

            if check_and_handle_pending_save(hwnd) {
                return LRESULT(0);
            }

            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            let mut should_close = false;
            {
                let mut session_guard = SESSION_PTR.lock().unwrap();
                if let Some(session) = session_guard.as_mut() {
                    if session.selection.is_none() {
                        session.outcome = CaptureOutcome::Cancelled;
                        *OUTCOME.lock().unwrap() = Some(CaptureOutcome::Cancelled);
                        should_close = true;
                    } else {
                        session.active_tool = crate::ui::toolbar::ToolType::Select;
                    }
                }
            }

            if should_close {
                let _ = DestroyWindow(hwnd);
            } else {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let mut session_guard = SESSION_PTR.lock().unwrap();
            if let Some(session) = session_guard.as_mut() {
                session.on_mouse_wheel(delta);
            }
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            let ctrl_down = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

            let mut close_window = false;
            {
                let mut session_guard = SESSION_PTR.lock().unwrap();
                if let Some(session) = session_guard.as_mut() {
                    if key == VK_ESCAPE.0 {
                        session.outcome = CaptureOutcome::Cancelled;
                        *OUTCOME.lock().unwrap() = Some(CaptureOutcome::Cancelled);
                        close_window = true;
                    } else if key == VK_RETURN.0 || (ctrl_down && key == b'C' as u16) {
                        session.copy_selection();
                        session.outcome = CaptureOutcome::CopiedToClipboard;
                        *OUTCOME.lock().unwrap() = Some(CaptureOutcome::CopiedToClipboard);
                        close_window = true;
                    } else if ctrl_down && key == b'S' as u16 {
                        session.trigger_action(crate::ui::toolbar::ActionType::Save);
                    } else if ctrl_down && key == b'Z' as u16 {
                        session.trigger_tool(crate::ui::toolbar::ToolType::Undo);
                    }
                }
            }

            if close_window {
                let _ = DestroyWindow(hwnd);
                return LRESULT(0);
            }

            if check_and_handle_pending_save(hwnd) {
                return LRESULT(0);
            }

            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Fallback for non-windows environments
#[cfg(not(windows))]
pub fn run_overlay_window(_screenshot: RgbaImage) -> Result<CaptureOutcome, String> {
    Err("Native UI overlay currently optimized for Windows DXGI/GDI architecture".to_string())
}

/// Creates a borderless floating always-on-top window pinned to the user's desktop.
#[cfg(windows)]
pub fn pin_to_desktop(image: RgbaImage) -> Result<(), String> {
    use windows::core::w;
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::Graphics::Gdi::HBRUSH;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;

    *PINNED_IMAGE.lock().unwrap() = Some(image.clone());

    let (w, h) = (image.width(), image.height());

    unsafe {
        let h_instance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW error: {e}"))?;
        let h_instance_typed = HINSTANCE(h_instance.0);
        let class_name = w!("HyperShotPinnedWindow");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(pinned_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance_typed,
            hIcon: crate::ui::daemon::load_tray_icon(),
            hCursor: LoadCursorW(None, IDC_SIZEALL).unwrap_or_default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: crate::ui::daemon::load_tray_icon(),
        };

        let _ = RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("HyperShot Pin"),
            WS_POPUP | WS_VISIBLE,
            100,
            100,
            w as i32,
            h as i32,
            None,
            None,
            Some(h_instance_typed),
            None,
        ).map_err(|e| format!("Failed to create pinned window: {e}"))?;

        let _ = ShowWindow(hwnd, SW_SHOW);
        info!("Pinned window active on desktop (double-click to close, drag to reposition)");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

#[cfg(windows)]
unsafe extern "system" fn pinned_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows::Win32::UI::WindowsAndMessaging::*;

    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let guard = PINNED_IMAGE.lock().unwrap();
            if let Some(img) = guard.as_ref() {
                let (w, h) = (img.width(), img.height());
                let bi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: w as i32,
                        biHeight: -(h as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let mut bgra = img.clone().into_raw();
                for chunk in bgra.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }

                let _ = SetDIBitsToDevice(
                    hdc,
                    0,
                    0,
                    w,
                    h,
                    0,
                    0,
                    0,
                    h,
                    bgra.as_ptr() as *const _,
                    &bi,
                    DIB_RGB_COLORS,
                );
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let _ = ReleaseCapture();
            let _ = SendMessageW(hwnd, WM_NCLBUTTONDOWN, Some(WPARAM(HTCAPTION as usize)), Some(lparam));
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK | WM_RBUTTONUP => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(not(windows))]
pub fn pin_to_desktop(_image: RgbaImage) -> Result<(), String> {
    Ok(())
}
