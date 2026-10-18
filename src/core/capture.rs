use image::RgbaImage;
use log::{debug, info, warn};
use xcap::Monitor;

/// Information about a connected display monitor.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

/// High-performance cross-platform screen capture engine.
///
/// On Windows: Utilizes DXGI Desktop Duplication API (DirectX 11) with instant fallback to Win32 GDI.
/// On macOS: Utilizes ScreenCaptureKit (macOS 12.3+) with fallback to CoreGraphics.
/// On Linux: Utilizes Wayland Desktop Portal (PipeWire) or X11 XShm.
pub struct CaptureEngine;

impl CaptureEngine {
    /// List all available monitors connected to the system.
    pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {e}"))?;
        let mut list = Vec::with_capacity(monitors.len());

        for (idx, m) in monitors.iter().enumerate() {
            let name = m.name().unwrap_or_else(|_| format!("Display {idx}"));
            let width = m.width().unwrap_or(0);
            let height = m.height().unwrap_or(0);
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);
            let is_primary = m.is_primary().unwrap_or(false);

            list.push(MonitorInfo {
                id: idx,
                name,
                width,
                height,
                x,
                y,
                is_primary,
            });
        }

        Ok(list)
    }

    /// Captures the entire desktop spanning all connected monitors (Virtual Desktop).
    pub fn capture_desktop() -> Result<RgbaImage, String> {
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN,
                SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            };

            unsafe {
                let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

                let x = if vw > 0 { vx } else { 0 };
                let y = if vh > 0 { vy } else { 0 };
                let width = if vw > 0 { vw as u32 } else { GetSystemMetrics(SM_CXSCREEN) as u32 };
                let height = if vh > 0 { vh as u32 } else { GetSystemMetrics(SM_CYSCREEN) as u32 };

                info!(
                    "Capturing full virtual desktop: ({}x{} at {}, {})",
                    width, height, x, y
                );

                match capture_via_gdi(x, y, width, height) {
                    Ok(img) => return Ok(img),
                    Err(e) => warn!("GDI virtual desktop capture failed ({e}), trying composite fallback..."),
                }
            }
        }

        Self::capture_all_monitors_composite()
    }

    /// Captures all monitors individually and composites them into a single image.
    pub fn capture_all_monitors_composite() -> Result<RgbaImage, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
        if monitors.is_empty() {
            return Err("No monitor found on the system".to_string());
        }

        if monitors.len() == 1 {
            let m = &monitors[0];
            return m.capture_image().map_err(|e| format!("Monitor capture failed: {e}"));
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        let mut mon_info = Vec::with_capacity(monitors.len());
        for m in &monitors {
            let mx = m.x().unwrap_or(0);
            let my = m.y().unwrap_or(0);
            let mw = m.width().unwrap_or(1920) as i32;
            let mh = m.height().unwrap_or(1080) as i32;

            min_x = min_x.min(mx);
            min_y = min_y.min(my);
            max_x = max_x.max(mx + mw);
            max_y = max_y.max(my + mh);

            mon_info.push((m, mx, my));
        }

        let total_w = (max_x - min_x).max(1) as u32;
        let total_h = (max_y - min_y).max(1) as u32;

        info!("Compositing {} monitors into {}x{} canvas", monitors.len(), total_w, total_h);

        let mut composite = RgbaImage::new(total_w, total_h);

        for (m, mx, my) in mon_info {
            if let Ok(img) = m.capture_image() {
                let dest_x = (mx - min_x) as i64;
                let dest_y = (my - min_y) as i64;
                image::imageops::overlay(&mut composite, &img, dest_x, dest_y);
            }
        }

        Ok(composite)
    }

    /// Captures the primary display monitor.
    #[allow(dead_code)]
    pub fn capture_primary() -> Result<RgbaImage, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;

        // Find primary monitor, or fallback to the first monitor
        let primary = monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .ok_or_else(|| "No monitor found on the system".to_string())?;

        let name = primary.name().unwrap_or_default();
        let width = primary.width().unwrap_or(1920);
        let height = primary.height().unwrap_or(1080);
        let x = primary.x().unwrap_or(0);
        let y = primary.y().unwrap_or(0);

        info!("Capturing primary display: {} ({}x{})", name, width, height);

        // Attempt 1: High-speed DirectX / native capture via xcap
        match primary.capture_image() {
            Ok(img) => {
                debug!("DirectX/GPU capture succeeded: {}x{} pixels", img.width(), img.height());
                Ok(img)
            }
            Err(e) => {
                warn!("DirectX capture unavailable ({e}), engaging Win32 GDI fallback...");
                #[cfg(windows)]
                {
                    capture_via_gdi(x, y, width, height)
                }
                #[cfg(not(windows))]
                {
                    Err(format!("Screen capture failed: {e}"))
                }
            }
        }
    }

    /// Captures a specific monitor by its index.
    #[allow(dead_code)]
    pub fn capture_monitor_by_index(index: usize) -> Result<RgbaImage, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
        let target = monitors
            .get(index)
            .ok_or_else(|| format!("Monitor index {index} is out of bounds"))?;

        let width = target.width().unwrap_or(1920);
        let height = target.height().unwrap_or(1080);
        let x = target.x().unwrap_or(0);
        let y = target.y().unwrap_or(0);

        match target.capture_image() {
            Ok(img) => Ok(img),
            Err(e) => {
                warn!("Monitor GPU capture returned {e}, switching to fallback");
                #[cfg(windows)]
                {
                    capture_via_gdi(x, y, width, height)
                }
                #[cfg(not(windows))]
                {
                    Err(format!("Monitor capture failed: {e}"))
                }
            }
        }
    }
}

/// Fallback capture via Win32 GDI BitBlt covering any monitor or the entire virtual screen.
#[cfg(windows)]
fn capture_via_gdi(x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage, String> {
    use windows::Win32::Graphics::Gdi::*;

    unsafe {
        // Passing None to GetDC retrieves the device context for the entire virtual screen
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err("Failed to get screen DC".to_string());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            ReleaseDC(None, hdc_screen);
            return Err("Failed to create compatible DC".to_string());
        }

        let h_bm = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
        if h_bm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("Failed to create compatible bitmap".to_string());
        }

        let old_bm = SelectObject(hdc_mem, h_bm.into());

        // Include CAPTUREBLT (0x40000000) to capture layered and transparent windows
        let rop = ROP_CODE(0x40000000 | SRCCOPY.0);

        let _ = BitBlt(
            hdc_mem,
            0,
            0,
            width as i32,
            height as i32,
            Some(hdc_screen),
            x,
            y,
            rop,
        );

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // Top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut raw_bytes = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            hdc_mem,
            h_bm,
            0,
            height,
            Some(raw_bytes.as_mut_ptr() as *mut _),
            &mut bi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI objects
        SelectObject(hdc_mem, old_bm);
        let _ = DeleteObject(h_bm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        if lines == 0 {
            return Err("GetDIBits failed to extract frame".to_string());
        }

        // Convert BGRA to RGBA and ensure full opacity
        for chunk in raw_bytes.chunks_exact_mut(4) {
            chunk.swap(0, 2);
            chunk[3] = 255;
        }

        RgbaImage::from_raw(width, height, raw_bytes)
            .ok_or_else(|| "Failed to construct RgbaImage from buffer".to_string())
    }
}
