#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::time::Instant;
use env_logger::Env;

mod core;
mod ui;

use crate::core::{
    annotator::{draw_arrow, draw_highlighter},
    capture::CaptureEngine,
    clipboard::copy_to_clipboard,
    counter::CounterManager,
    filters::{pixelate, Rect},
};
use crate::ui::{
    find_and_signal_running_instance, run_tray_daemon,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // On Windows, attach to parent console if started from CMD or PowerShell
    #[cfg(windows)]
    unsafe {
        let _ = windows::Win32::System::Console::AttachConsole(
            windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    let is_demo_mode = args.iter().any(|arg| arg == "--demo");
    let is_daemon_mode = args.iter().any(|arg| arg == "--daemon" || arg == "--tray");
    let is_help_mode = args.iter().any(|arg| arg == "--help" || arg == "-h");

    if is_help_mode {
        println!("============================================================");
        println!("  ⚡ HyperShot - Next-Gen Screenshot & Annotation Engine");
        println!("  Speed of Lightshot + Power of Flameshot + Scroll Capture");
        println!("============================================================\n");
        println!("Usage: hypershot [OPTIONS]\n");
        println!("Options:");
        println!("  --daemon, --tray   Run silently in background system tray (Windows Startup)");
        println!("  --demo             Run high-speed capture benchmark and save demo image");
        println!("  --help, -h         Show this help information\n");
        println!("Default Behavior:");
        println!("  • If already running: Triggers screenshot overlay immediately");
        println!("  • If not running: Starts background tray and triggers screenshot overlay");
        println!("  • Global Hotkeys: [PrintScreen] or [Ctrl + Shift + X]\n");
        return Ok(());
    }

    if is_demo_mode {
        run_benchmark_demo()?;
        return Ok(());
    }

    // 1. If another HyperShot instance (tray daemon) is already running:
    // Notify the existing instance to trigger screenshot overlay immediately!
    #[cfg(windows)]
    {
        if find_and_signal_running_instance() {
            println!("⚡ Notified existing HyperShot instance to capture screen.");
            return Ok(());
        }
    }

    // 2. If launched with --daemon or --tray (e.g., Windows Autostart on boot):
    if is_daemon_mode {
        run_tray_daemon(false)?;
        return Ok(());
    }

    // 3. Normal launch from Desktop or Start Menu:
    // Starts background system tray daemon AND triggers screenshot overlay right away!
    println!("⚡ Starting HyperShot with system tray and instant capture...");
    run_tray_daemon(true)?;
    Ok(())
}

/// Runs non-interactive validation demo and saves sample image
fn run_benchmark_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  ⚡ HyperShot - Next-Gen Screenshot & Annotation Engine");
    println!("  Speed of Lightshot + Power of Flameshot + Scroll Capture");
    println!("============================================================\n");

    let monitors = CaptureEngine::list_monitors()?;
    println!("🖥️  Connected Displays ({}):", monitors.len());
    for m in &monitors {
        let primary_badge = if m.is_primary { " [PRIMARY]" } else { "" };
        println!(
            "   • [{}] {} - {}x{} at ({}, {}){}",
            m.id, m.name, m.width, m.height, m.x, m.y, primary_badge
        );
    }
    println!();

    print!("📸 Performing high-speed multi-monitor capture benchmark... ");
    let t0 = Instant::now();
    let mut screenshot = CaptureEngine::capture_desktop()?;
    let elapsed = t0.elapsed();
    println!("OK! ({:.2} ms)", elapsed.as_secs_f64() * 1000.0);

    let center_x = screenshot.width() / 2;
    let center_y = screenshot.height() / 2;

    let pixelate_rect = Rect::new(center_x.saturating_sub(180), center_y.saturating_sub(40), 160, 70);
    pixelate(&mut screenshot, pixelate_rect, 8);

    draw_arrow(
        &mut screenshot,
        center_x as i32 + 100,
        center_y as i32 - 100,
        pixelate_rect.x as i32 + (pixelate_rect.width as i32 / 2),
        pixelate_rect.y as i32 + (pixelate_rect.height as i32 / 2),
        [235, 47, 47, 255],
        4,
    );

    let highlight_rect = Rect::new(center_x.saturating_sub(180), center_y + 50, 200, 30);
    draw_highlighter(&mut screenshot, highlight_rect, [255, 230, 0, 255]);

    let mut counter = CounterManager::new();
    counter.add_step(pixelate_rect.x, pixelate_rect.y.saturating_sub(20));
    counter.add_step(highlight_rect.x, highlight_rect.y.saturating_sub(20));
    counter.render_all(&mut screenshot);

    let output_path = "hypershot_demo.png";
    screenshot.save(output_path)?;
    println!("💾 Saved demonstration image to: {}", output_path);

    let _ = copy_to_clipboard(&screenshot);
    println!("📋 Screenshot copied to OS clipboard!");
    Ok(())
}
