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
use crate::ui::{pin_to_desktop, run_overlay_window, CaptureOutcome};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    let is_demo_mode = args.iter().any(|arg| arg == "--demo");

    println!("============================================================");
    println!("  ⚡ HyperShot - Next-Gen Screenshot & Annotation Engine");
    println!("  Speed of Lightshot + Power of Flameshot + Scroll Capture");
    println!("============================================================\n");

    // 1. Enumerate displays
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

    if is_demo_mode {
        run_benchmark_demo()?;
        return Ok(());
    }

    // 2. Perform instant high-speed screen capture
    print!("📸 Capturing desktop frame... ");
    let t0 = Instant::now();
    let screenshot = CaptureEngine::capture_primary()?;
    let elapsed = t0.elapsed();
    println!("OK! ({:.2} ms)", elapsed.as_secs_f64() * 1000.0);
    println!("   Resolution: {}x{} pixels", screenshot.width(), screenshot.height());

    println!("\n🎯 Launching HyperShot Interactive Overlay...");
    println!("   • Click & Drag to select rectangle");
    println!("   • Right Toolbar: Pen, Line, Arrow, Rect, Highlighter, Pixelate, Step Badges ①②③, Colors");
    println!("   • Bottom Toolbar: Copy (Ctrl+C), Save (Ctrl+S), Pin to Desktop, Close (Esc)");
    println!("   • Loupe & Color Picker: Active before selection\n");

    // 3. Launch interactive overlay window
    match run_overlay_window(screenshot)? {
        CaptureOutcome::CopiedToClipboard => {
            println!("📋 Selection copied directly to OS clipboard! Ready to paste (Ctrl+V).");
        }
        CaptureOutcome::SavedToFile(path) => {
            println!("💾 Screenshot saved successfully to: {}", path);
        }
        CaptureOutcome::PinnedToDesktop(crop) => {
            println!("📌 Pinned screenshot to desktop (Always on Top).");
            let _ = pin_to_desktop(crop);
        }
        CaptureOutcome::Cancelled => {
            println!("❌ Screenshot capture cancelled.");
        }
    }

    Ok(())
}

/// Runs non-interactive validation demo and saves sample image
fn run_benchmark_demo() -> Result<(), Box<dyn std::error::Error>> {
    print!("📸 Performing high-speed screen capture benchmark... ");
    let t0 = Instant::now();
    let mut screenshot = CaptureEngine::capture_primary()?;
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
