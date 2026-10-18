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

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(daemon_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance_typed,
            hIcon: HICON::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
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
            hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
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

#[cfg(not(windows))]
pub fn run_tray_daemon(_trigger_immediately: bool) -> Result<(), String> {
    Err("Daemon currently implemented for Windows".to_string())
}

#[cfg(not(windows))]
pub fn find_and_signal_running_instance() -> bool {
    false
}
