use std::time::Instant;
use env_logger::Env;

mod core;
use crate::core::{
    capture::CaptureEngine,
    filters::{pixelate, Rect},
    counter::CounterManager,
    annotator::{draw_arrow, draw_highlighter},
    clipboard::copy_to_clipboard,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    println!("============================================================");
    println!("  ⚡ HyperShot - Next-Gen Screenshot & Annotation Engine");
    println!("  Speed of Lightshot + Power of Flameshot + Scroll Capture");
    println!("============================================================\n");

    // 1. Enumerate and display connected displays
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

    // 2. Perform zero-latency screen capture benchmark
    print!("📸 Performing high-speed screen capture... ");
    let t0 = Instant::now();
    let mut screenshot = CaptureEngine::capture_primary()?;
    let elapsed = t0.elapsed();
    println!("OK! ({:.2} ms)", elapsed.as_secs_f64() * 1000.0);
    println!("   Resolution: {}x{} pixels", screenshot.width(), screenshot.height());

    // 3. Demonstrate core filter & annotation pipelines on a sample region
    println!("\n🎨 Demonstrating Core Annotator & Privacy Pipelines:");
    let center_x = screenshot.width() / 2;
    let center_y = screenshot.height() / 2;

    // A. Pixelate a 160x70 region (Privacy blur for passwords/credentials)
    let pixelate_rect = Rect::new(center_x.saturating_sub(180), center_y.saturating_sub(40), 160, 70);
    pixelate(&mut screenshot, pixelate_rect, 8);
    println!("   ✓ Pixelate filter applied (block_size: 8px)");

    // B. Draw an arrow pointing to the censored region
    draw_arrow(
        &mut screenshot,
        center_x as i32 + 100,
        center_y as i32 - 100,
        pixelate_rect.x as i32 + (pixelate_rect.width as i32 / 2),
        pixelate_rect.y as i32 + (pixelate_rect.height as i32 / 2),
        [235, 47, 47, 255], // Red
        4,
    );
    println!("   ✓ Directional arrow rendered");

    // C. Draw fluorescent highlighter
    let highlight_rect = Rect::new(center_x.saturating_sub(180), center_y + 50, 200, 30);
    draw_highlighter(&mut screenshot, highlight_rect, [255, 230, 0, 255]); // Yellow highlighter
    println!("   ✓ Fluorescent highlighter applied");

    // D. Add step counters (①, ②, ③)
    let mut counter = CounterManager::new();
    counter.add_step(pixelate_rect.x, pixelate_rect.y.saturating_sub(20));
    counter.add_step(highlight_rect.x, highlight_rect.y.saturating_sub(20));
    counter.render_all(&mut screenshot);
    println!("   ✓ Step badges ① and ② injected");

    // 4. Save demo artifact
    let output_path = "hypershot_demo.png";
    screenshot.save(output_path)?;
    println!("\n💾 Saved demonstration image to: {}", output_path);

    // 5. Test clipboard integration
    if let Err(e) = copy_to_clipboard(&screenshot) {
        println!("⚠️  Clipboard notice: {}", e);
    } else {
        println!("📋 Screenshot copied to OS clipboard! (Press Ctrl+V to paste)");
    }

    println!("\n🚀 HyperShot core engine is fully operational!");
    Ok(())
}
