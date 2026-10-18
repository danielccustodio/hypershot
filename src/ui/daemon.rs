use log::info;
use crate::core::capture::CaptureEngine;
use crate::ui::window::run_overlay_window;
use crate::ui::overlay::CaptureOutcome;

#[cfg(windows)]
pub const WM_TRIGGER_CAPTURE: u32 = windows::Win32::UI::WindowsAndMessaging::WM_USER + 102;

#[cfg(windows)]
pub fn find_and_signal_running_instance() -> bool {
    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW};

    unsafe {
        if let Ok(hwnd) = FindWindowW(w!("HyperShotDaemonClass"), None) {
            if !hwnd.0.is_null() {
                let _ = PostMessageW(
                    Some(hwnd),
                    WM_TRIGGER_CAPTURE,
                    windows::Win32::Foundation::WPARAM(0),
                    windows::Win32::Foundation::LPARAM(0),
                );
                return true;
            }
        }
    }
    false
}

#[cfg(windows)]
pub fn run_tray_daemon(trigger_immediately: bool) -> Result<(), String> {
    use windows::core::w;
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_SHIFT, VK_SNAPSHOT,
    };
    use windows::Win32::UI::Shell::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAYICON: u32 = WM_USER + 101;
    const HOTKEY_PRINTSCREEN: i32 = 1001;
    const HOTKEY_CTRL_SHIFT_X: i32 = 1002;

    unsafe {
        let h_instance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW error: {e}"))?;
        let h_instance_typed = HINSTANCE(h_instance.0);
        let class_name = w!("HyperShotDaemonClass");

        let app_icon = load_tray_icon();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(daemon_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance_typed,
            hIcon: app_icon,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: app_icon,
        };

        let _ = RegisterClassExW(&wnd_class);

        // Invisible message-only window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("HyperShot Daemon"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            None,
            None,
            Some(h_instance_typed),
            None,
        ).map_err(|e| format!("Failed to create daemon window: {e}"))?;

        // 1. Register Global Hotkeys
        let hk1 = RegisterHotKey(Some(hwnd), HOTKEY_PRINTSCREEN, HOT_KEY_MODIFIERS::default(), VK_SNAPSHOT.0 as u32);
        if hk1.is_err() {
            eprintln!("Notice: PrintScreen hotkey could not be exclusively registered. Registering Ctrl+Shift+X fallback...");
        } else {
            info!("Registered global hotkey: [PrintScreen]");
        }

        let _ = RegisterHotKey(Some(hwnd), HOTKEY_CTRL_SHIFT_X, MOD_CONTROL | MOD_SHIFT, b'X' as u32);
        info!("Registered global hotkey: [Ctrl + Shift + X]");

        // 2. Initialize System Tray Icon
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: app_icon,
            ..Default::default()
        };

        let tip_str = "HyperShot (PrintScreen / Ctrl+Shift+X)\0";
        for (i, c) in tip_str.encode_utf16().enumerate() {
            if i < nid.szTip.len() {
                nid.szTip[i] = c;
            }
        }

        let _ = Shell_NotifyIconW(NIM_ADD, &nid);

        println!("⚡ HyperShot Daemon is running in background!");
        println!("   • Press [PrintScreen] or [Ctrl + Shift + X] to capture");
        println!("   • Check taskbar notification tray icon for menu\n");

        if trigger_immediately {
            trigger_screenshot();
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        let _ = UnregisterHotKey(Some(hwnd), HOTKEY_PRINTSCREEN);
        let _ = UnregisterHotKey(Some(hwnd), HOTKEY_CTRL_SHIFT_X);
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }

    Ok(())
}

#[cfg(windows)]
unsafe extern "system" fn daemon_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::core::w;
    use windows::Win32::Foundation::{LRESULT, POINT};
    use windows::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAYICON: u32 = WM_USER + 101;
    const MENU_CAPTURE: usize = 2001;
    const MENU_FOLDER: usize = 2002;
    const MENU_SETTINGS: usize = 2003;
    const MENU_EXIT: usize = 2004;

    match msg {
        WM_HOTKEY | WM_TRIGGER_CAPTURE => {
            trigger_screenshot();
            LRESULT(0)
        }
        WM_TRAYICON => {
            let event = (lparam.0 & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP => {
                    trigger_screenshot();
                }
                WM_RBUTTONUP => {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    let h_menu = CreatePopupMenu().unwrap_or_default();
                    let _ = AppendMenuW(h_menu, MF_STRING, MENU_CAPTURE, w!("⚡ Capturar Tela (PrintScreen)"));
                    let _ = AppendMenuW(h_menu, MF_SEPARATOR, 0, windows::core::PCWSTR::null());
                    let _ = AppendMenuW(h_menu, MF_STRING, MENU_FOLDER, w!("📁 Abrir Pasta de Capturas"));
                    let _ = AppendMenuW(h_menu, MF_STRING, MENU_SETTINGS, w!("⚙️ Editar Preferências (config.json)"));
                    let _ = AppendMenuW(h_menu, MF_SEPARATOR, 0, windows::core::PCWSTR::null());
                    let _ = AppendMenuW(h_menu, MF_STRING, MENU_EXIT, w!("❌ Sair do HyperShot"));

                    let _ = SetForegroundWindow(hwnd);
                    let cmd = TrackPopupMenu(
                        h_menu,
                        TPM_RETURNCMD | TPM_NONOTIFY,
                        pt.x,
                        pt.y,
                        Some(0),
                        hwnd,
                        None,
                    );
                    let _ = DestroyMenu(h_menu);

                    match cmd.0 as usize {
                        MENU_CAPTURE => trigger_screenshot(),
                        MENU_FOLDER => {
                            let cfg = crate::core::config::AppConfig::load();
                            let _ = std::process::Command::new("explorer").arg(&cfg.save_directory).spawn();
                        }
                        MENU_SETTINGS => {
                            if let Some(path) = crate::core::config::AppConfig::config_path() {
                                let _ = std::process::Command::new("notepad").arg(path).spawn();
                            }
                        }
                        MENU_EXIT => {
                            let _ = DestroyWindow(hwnd);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn trigger_screenshot() {
    println!("📸 Triggering instant capture...");
    match CaptureEngine::capture_desktop() {
        Ok(screenshot) => {
            println!("   Captured {}x{} multi-monitor frame. Opening overlay...", screenshot.width(), screenshot.height());
            if let Ok(outcome) = run_overlay_window(screenshot) {
                match outcome {
                    CaptureOutcome::CopiedToClipboard => println!("📋 Copied to clipboard!"),
                    CaptureOutcome::SavedToFile(p) => println!("💾 Saved to {}", p),
                    CaptureOutcome::PinnedToDesktop(img) => {
                        println!("📌 Pinning to desktop...");
                        let _ = crate::ui::window::pin_to_desktop(img);
                    }
                    CaptureOutcome::Cancelled => println!("Cancelled."),
                }
            }
        }
        Err(e) => eprintln!("Capture error: {}", e),
    }
}

#[cfg(windows)]
pub fn load_tray_icon() -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject};
    use windows::Win32::UI::WindowsAndMessaging::*;

    // 1. Try loading icon.ico from executable's directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let ico_path = dir.join("icon.ico");
            if ico_path.exists() {
                let wide: Vec<u16> = ico_path.to_string_lossy().encode_utf16().chain(Some(0)).collect();
                if let Ok(handle) = unsafe {
                    LoadImageW(None, PCWSTR(wide.as_ptr()), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE)
                } {
                    if !handle.is_invalid() {
                        return HICON(handle.0);
                    }
                }
            }
        }
    }

    // 2. Try loading from working directory candidate paths
    for candidate in &["assets/icon.ico", "icon.ico", "../assets/icon.ico"] {
        let p = std::path::Path::new(candidate);
        if p.exists() {
            let wide: Vec<u16> = p.to_string_lossy().encode_utf16().chain(Some(0)).collect();
            if let Ok(handle) = unsafe {
                LoadImageW(None, PCWSTR(wide.as_ptr()), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE)
            } {
                if !handle.is_invalid() {
                    return HICON(handle.0);
                }
            }
        }
    }

    // 3. Fallback: High-contrast 32-bit ARGB tray icon with glowing cyan reticle and electric amber bolt
    unsafe {
        let mut pixels = [0u32; 32 * 32];
        let mask_bits = [0u8; 32 * 4];

        // Background: rounded dark slate squircle
        for y in 2..=29 {
            for x in 2..=29 {
                let is_corner = (x <= 4 && y <= 4)
                    || (x >= 27 && y <= 4)
                    || (x <= 4 && y >= 27)
                    || (x >= 27 && y >= 27);
                if !is_corner {
                    pixels[y * 32 + x] = 0xFF0F172A; // Deep dark slate
                }
            }
        }

        // Cyan reticle corner brackets (thickness 2px)
        let cyan: u32 = 0xFF00E5FF;
        for i in 5..=11 {
            // Top-left
            pixels[5 * 32 + i] = cyan;
            pixels[6 * 32 + i] = cyan;
            pixels[i * 32 + 5] = cyan;
            pixels[i * 32 + 6] = cyan;
            // Top-right
            pixels[5 * 32 + (31 - i)] = cyan;
            pixels[6 * 32 + (31 - i)] = cyan;
            pixels[i * 32 + 25] = cyan;
            pixels[i * 32 + 26] = cyan;
            // Bottom-left
            pixels[25 * 32 + i] = cyan;
            pixels[26 * 32 + i] = cyan;
            pixels[(31 - i) * 32 + 5] = cyan;
            pixels[(31 - i) * 32 + 6] = cyan;
            // Bottom-right
            pixels[25 * 32 + (31 - i)] = cyan;
            pixels[26 * 32 + (31 - i)] = cyan;
            pixels[(31 - i) * 32 + 25] = cyan;
            pixels[(31 - i) * 32 + 26] = cyan;
        }

        // Electric lightning bolt in vibrant amber & white core
        let amber: u32 = 0xFFFFB300;
        let white: u32 = 0xFFFFFFFF;
        let bolt_points = [
            (17, 7), (18, 7),
            (16, 8), (17, 8), (18, 8),
            (15, 9), (16, 9), (17, 9),
            (14, 10), (15, 10), (16, 10),
            (13, 11), (14, 11), (15, 11),
            (12, 12), (13, 12), (14, 12),
            (11, 13), (12, 13), (13, 13), (14, 13), (15, 13), (16, 13), (17, 13), (18, 13),
            (13, 14), (14, 14), (15, 14), (16, 14), (17, 14), (18, 14), (19, 14),
            (14, 15), (15, 15), (16, 15), (17, 15),
            (15, 16), (16, 16), (17, 16),
            (16, 17), (17, 17), (18, 17),
            (17, 18), (18, 18),
            (18, 19), (19, 19),
            (19, 20), (20, 20),
            (20, 21), (21, 21),
            (21, 22),
            (21, 23),
            (22, 24),
        ];

        for &(bx, by) in &bolt_points {
            if bx < 32 && by < 32 {
                pixels[by * 32 + bx] = amber;
            }
        }
        // White core inside lightning
        for y in 10..=16 {
            pixels[y * 32 + 15] = white;
            pixels[y * 32 + 16] = white;
        }

        let hbm_color = CreateBitmap(32, 32, 1, 32, Some(pixels.as_ptr() as *const _));
        let hbm_mask = CreateBitmap(32, 32, 1, 1, Some(mask_bits.as_ptr() as *const _));

        let icon_info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let hicon = CreateIconIndirect(&icon_info);
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_color.0));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_mask.0));

        if let Ok(icon) = hicon {
            if !icon.is_invalid() {
                return icon;
            }
        }

        LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
    }
}

#[cfg(not(windows))]
pub fn run_tray_daemon(_trigger_immediately: bool) -> Result<(), String> {
    Err("Daemon currently implemented for Windows".to_string())
}

#[cfg(not(windows))]
pub fn find_and_signal_running_instance() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_load_tray_icon_returns_valid_handle() {
        let icon = load_tray_icon();
        assert!(!icon.is_invalid(), "Tray icon must not be an invalid handle!");
    }
}

