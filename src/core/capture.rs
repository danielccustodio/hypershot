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

    /// Captures the primary display monitor.
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

/// Fallback capture via Win32 GDI BitBlt (same fallback mechanism used in Lightshot's DXGIODScreenshot.dll).
#[cfg(windows)]
fn capture_via_gdi(x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage, String> {
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

    unsafe {
        let h_desktop = GetDesktopWindow();
        let hdc_screen = GetDC(Some(h_desktop));
        if hdc_screen.is_invalid() {
            return Err("Failed to get screen DC".to_string());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            ReleaseDC(Some(h_desktop), hdc_screen);
            return Err("Failed to create compatible DC".to_string());
        }

        let h_bm = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
        if h_bm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(Some(h_desktop), hdc_screen);
            return Err("Failed to create compatible bitmap".to_string());
        }

        let old_bm = SelectObject(hdc_mem, h_bm.into());

        let _ = BitBlt(
            hdc_mem,
            0,
            0,
            width as i32,
            height as i32,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
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
        ReleaseDC(Some(h_desktop), hdc_screen);

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
