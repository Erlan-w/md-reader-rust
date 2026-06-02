#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod install;
mod markdown;
mod template;

use std::{
    env,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;



fn main() {
    // ── Subcommand dispatch ─────────────────────────────────────────────────
    match env::args().nth(1).as_deref() {
        Some("install") => return install::install(),
        Some("uninstall") => return install::uninstall(),
        _ => {}
    }

    // ── Parse CLI argument ──────────────────────────────────────────────────
    let md_path: PathBuf = match env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            // No argument → open native file dialog
            match open_file_dialog() {
                Some(p) => p,
                None => return,
            }
        }
    };

    if !md_path.exists() {
        eprintln!("File not found: {}", md_path.display());
        std::process::exit(1);
    }

    // ── Build HTML from Markdown ────────────────────────────────────────────
    let filename = md_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let html = match build_page(&md_path) {
        Ok(h) => h,
        Err(e) => {
            log_error(&format!("Failed to build page: {e}"));
            return;
        }
    };

    // ── Write to cache file (needed so local file:// works for Mermaid) ─────
    let cache_path = match write_cache(&html) {
        Ok(p) => p,
        Err(e) => {
            log_error(&format!("Failed to write cache: {e}"));
            return;
        }
    };

    let url = format!("file://{}", cache_path.to_string_lossy().replace('\\', "/"));

    // ── Create window & webview ─────────────────────────────────────────────
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title(format!("MD Reader — {filename}"))
        .with_inner_size(LogicalSize::new(1200u32, 800u32))
        .with_min_inner_size(LogicalSize::new(800u32, 500u32))
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    // Pre-load URL
    let _webview = WebViewBuilder::new(&window)
        .with_url(&url)
        .with_background_color((10, 11, 15, 255))
        .build()
        .expect("Failed to build WebView");

    // ── Event loop ──────────────────────────────────────────────────────────
    let cache_path_arc = Arc::new(Mutex::new(cache_path));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            // Clean up cache file on close
            if let Ok(path) = cache_path_arc.lock() {
                let _ = fs::remove_file(&*path);
            }
            *control_flow = ControlFlow::Exit;
        }
    });
}

/// Open native OS file picker dialog.
fn open_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .add_filter("All files", &["*"])
        .set_title("Open Markdown File")
        .pick_file()
}

/// Build a complete HTML page from a Markdown file path.
fn build_page(md_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(md_path)?;
    let filename = md_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let (html_content, word_count) = markdown::render(&raw);
    let read_time = (word_count as f64 / 200.0).ceil() as usize;

    let mermaid_src = get_mermaid_src(md_path);
    let page = template::HTML_TEMPLATE
        .replace("__CONTENT__", &html_content)
        .replace("__FILENAME__", &html_escape(&filename))
        .replace("__MERMAID_SRC__", &mermaid_src)
        .replace("__WORD_COUNT__", &format!("{}", word_count))
        .replace("__READ_TIME__", &format!("{}", read_time))
        .replace("__VERSION__", env!("CARGO_PKG_VERSION"));

    Ok(page)
}

/// Escape special HTML characters for safe injection into attributes.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Locate mermaid.min.js: first try local dist/, then fall back to CDN.
fn get_mermaid_src(md_path: &Path) -> String {
    // 1. Next to the binary
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let local = dir.join("dist").join("mermaid.min.js");
            if local.exists() {
                return file_url(&local);
            }
            // 1b. Development: target/{debug,release}/ → ../../src/dist/
            let dev = dir.join("../../src/dist").join("mermaid.min.js");
            if dev.exists() {
                return file_url(&dev);
            }
        }
    }

    // 2. Next to the .md file
    if let Some(dir) = md_path.parent() {
        let local = dir.join("dist").join("mermaid.min.js");
        if local.exists() {
            return file_url(&local);
        }
    }

    // 3. User data dir
    if let Some(data_dir) = dirs::data_local_dir() {
        let local = data_dir
            .join("MDReader")
            .join("dist")
            .join("mermaid.min.js");
        if local.exists() {
            return file_url(&local);
        }
    }

    // 4. Fallback CDN (defensive — never reached after install)
    "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js".to_string()
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy().replace('\\', "/"))
}

/// Write the HTML to a temp cache file and return its path.
fn write_cache(html: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MDReader")
        .join("cache");

    fs::create_dir_all(&cache_dir)?;

    let path = cache_dir.join("content.html");
    fs::write(&path, html.as_bytes())?;
    Ok(path)
}

/// Write an error message to the MDReader error log.
fn log_error(msg: &str) {
    if let Some(data_dir) = dirs::data_local_dir() {
        let log_dir = data_dir.join("MDReader");
        let _ = fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("error.log");
        let line = format!("{msg}\n");
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let _ = f.write_all(line.as_bytes());
        }
    }
    eprintln!("{msg}");
}