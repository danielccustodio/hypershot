# Contributing to HyperShot ⚡

Thank you for your interest in contributing to **HyperShot**! We are committed to building the fastest, most reliable open-source screenshot, annotation, and scroll-capture tool for Windows, macOS, and Linux.

---

## 🛠️ Development Setup

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (version 1.75+)
- Git

### Build & Run
```bash
# Clone the repository
git clone https://github.com/danielcustodio/hypershot.git
cd hypershot

# Run automated unit tests
cargo test

# Build and run in development mode
cargo run

# Build high-performance release binary
cargo build --release
```

---

## 📐 Project Structure

- `src/main.rs`: Application entry point and CLI commands.
- `src/core/capture.rs`: Multi-monitor screen capture engine (DirectX DXGI / ScreenCaptureKit / Wayland / X11).
- `src/core/filters.rs`: Privacy filters (pixelate mosaic and box/Gaussian blur).
- `src/core/counter.rs`: Auto-incrementing step sequence badges (①, ②, ③).
- `src/core/annotator.rs`: Drawing tools (arrows, rectangles, translucent highlighters).
- `src/core/stitching.rs`: Continuous scroll capture and image stitching engine.
- `src/core/clipboard.rs`: Zero-copy image clipboard synchronization.

---

## 🧪 Code Quality & Guidelines

1. **Performance First**: Avoid unnecessary heap allocations or CPU blocking inside hot loops (e.g. screen capture frames or pixel loops).
2. **Cross-Platform Compatibility**: Every PR should compile cleanly on Windows, Linux, and macOS.
3. **Format & Lint**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features
   ```
4. **Tests**: Add unit tests for any new algorithms, image filters, or stitching logic.

---

## 🤝 Submitting a Pull Request

1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/my-amazing-feature
   ```
2. Commit your changes with clear, descriptive commit messages.
3. Ensure all tests pass (`cargo test`).
4. Push to your branch and open a Pull Request.
