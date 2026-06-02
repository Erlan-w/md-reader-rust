use std::path::{Path, PathBuf};
use std::{env, fs};

// ── Public API ─────────────────────────────────────────────────────────────────

pub fn install() {
    let exe_path = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            show_error(&format!("Cannot locate binary: {e}"));
            return;
        }
    };
    let exe_dir = exe_path.parent().expect("binary has no parent dir");
    let install_dir = install_dir();
    let exe_name = exe_path.file_name().expect("binary has no filename");

    // 1. Create install directory
    if let Err(e) = fs::create_dir_all(&install_dir) {
        show_error(&format!("Cannot create install directory: {e}"));
        return;
    }

    // 2. Copy binary
    let dest_exe = install_dir.join(exe_name);
    if let Err(e) = fs::copy(&exe_path, &dest_exe) {
        show_error(&format!("Failed to copy binary: {e}"));
        return;
    }

    // 3. Copy offline assets
    copy_mermaid(exe_dir, &install_dir);
    copy_icon(exe_dir, &install_dir);

    // 4. Platform-specific registration
    #[cfg(target_os = "windows")]
    platform::install(&install_dir, &dest_exe);

    #[cfg(target_os = "linux")]
    platform::install(&dest_exe);

    #[cfg(target_os = "macos")]
    platform::install(&dest_exe);

    show_success("MD Reader installed successfully!");
}

pub fn uninstall() {
    // 1. Platform-specific unregistration
    #[cfg(target_os = "windows")]
    platform::uninstall();

    #[cfg(target_os = "linux")]
    platform::uninstall();

    #[cfg(target_os = "macos")]
    platform::uninstall();

    // 2. Remove all installed files
    let install_dir = install_dir();
    if install_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&install_dir) {
            show_error(&format!("Failed to remove files: {e}"));
            return;
        }
    }

    show_success("MD Reader uninstalled successfully!");
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

fn install_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MDReader")
}

fn copy_mermaid(exe_dir: &Path, install_dir: &Path) {
    let candidates = [
        // Standard: exe_dir/dist/mermaid.min.js
        exe_dir.join("dist").join("mermaid.min.js"),
        // Development: target/{debug,release}/ → ../../src/dist/
        exe_dir.join("../../src/dist").join("mermaid.min.js"),
    ];

    let dest_dir = install_dir.join("dist");
    let _ = fs::create_dir_all(&dest_dir);

    for src in &candidates {
        if src.exists() {
            if fs::copy(src, dest_dir.join("mermaid.min.js")).is_ok() {
                return;
            }
        }
    }
}

fn copy_icon(exe_dir: &Path, install_dir: &Path) {
    let candidates = [
        // Standard: exe_dir/icon/favicon.ico
        exe_dir.join("icon").join("favicon.ico"),
        // Development: target/{debug,release}/ → ../../src/icon/
        exe_dir.join("../../src/icon").join("favicon.ico"),
    ];

    for src in &candidates {
        if src.exists() {
            if fs::copy(src, install_dir.join("favicon.ico")).is_ok() {
                return;
            }
        }
    }
}

fn show_message(title: &str, msg: &str, level: rfd::MessageLevel) {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(msg)
        .set_level(level)
        .show();
}

fn show_error(msg: &str) {
    show_message("MD Reader - Error", msg, rfd::MessageLevel::Error);
}

fn show_success(msg: &str) {
    show_message("MD Reader", msg, rfd::MessageLevel::Info);
}

// ── Windows ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use std::os::windows::process::CommandExt;
    use std::path::Path;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    pub fn install(install_dir: &Path, exe_path: &Path) {
        let exe = exe_path.to_string_lossy().replace('/', "\\");
        let dir = install_dir.to_string_lossy().replace('/', "\\");
        let cmd_val = format!("\"{exe}\" \"%1\"");

        // Applications\md-reader
        reg(&[
            "add", "HKCU\\Software\\Classes\\Applications\\md-reader",
            "/ve", "/t", "REG_SZ", "/d", "MD Reader", "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Classes\\Applications\\md-reader\\shell\\open\\command",
            "/ve", "/t", "REG_SZ", "/d", &cmd_val, "/f",
        ]);

        // MDReader.Markdown (ProgID)
        reg(&[
            "add", "HKCU\\Software\\Classes\\MDReader.Markdown",
            "/ve", "/t", "REG_SZ", "/d", "MD Reader Markdown Document", "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Classes\\MDReader.Markdown\\DefaultIcon",
            "/ve", "/t", "REG_SZ", "/d", &format!("\"{exe}\",0"), "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Classes\\MDReader.Markdown\\shell\\open\\command",
            "/ve", "/t", "REG_SZ", "/d", &cmd_val, "/f",
        ]);

        // .md → MDReader.Markdown
        reg(&[
            "add", "HKCU\\Software\\Classes\\.md\\OpenWithProgids",
            "/v", "MDReader.Markdown", "/t", "REG_NONE", "/d", "", "/f",
        ]);
        // .markdown → MDReader.Markdown
        reg(&[
            "add", "HKCU\\Software\\Classes\\.markdown\\OpenWithProgids",
            "/v", "MDReader.Markdown", "/t", "REG_NONE", "/d", "", "/f",
        ]);

        // InstallPath
        reg(&[
            "add", "HKCU\\Software\\MDReader",
            "/v", "InstallPath", "/t", "REG_SZ", "/d", &dir, "/f",
        ]);

        // Control Panel uninstall entry
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "DisplayName", "/t", "REG_SZ", "/d", "MD Reader", "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "UninstallString", "/t", "REG_SZ",
            "/d", &format!("\"{exe}\" uninstall"), "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "DisplayIcon", "/t", "REG_SZ",
            "/d", &format!("\"{exe}\",0"), "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "DisplayVersion", "/t", "REG_SZ",
            "/d", env!("CARGO_PKG_VERSION"), "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "Publisher", "/t", "REG_SZ", "/d", "MD Reader Contributors", "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "InstallLocation", "/t", "REG_SZ", "/d", &dir, "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "NoModify", "/t", "REG_DWORD", "/d", "1", "/f",
        ]);
        reg(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader",
            "/v", "NoRepair", "/t", "REG_DWORD", "/d", "1", "/f",
        ]);

        // Start Menu shortcut
        create_shortcut(exe_path);
    }

    pub fn uninstall() {
        // Remove Control Panel entry
        reg(&["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MDReader", "/f"]);

        // Remove file associations
        reg(&["delete", "HKCU\\Software\\Classes\\.md\\OpenWithProgids", "/v", "MDReader.Markdown", "/f"]);
        reg(&["delete", "HKCU\\Software\\Classes\\.markdown\\OpenWithProgids", "/v", "MDReader.Markdown", "/f"]);
        reg(&["delete", "HKCU\\Software\\Classes\\MDReader.Markdown", "/f"]);
        reg(&["delete", "HKCU\\Software\\Classes\\Applications\\md-reader", "/f"]);
        reg(&["delete", "HKCU\\Software\\MDReader", "/f"]);

        // Remove Start Menu shortcut
        remove_shortcut();
    }

    fn reg(args: &[&str]) {
        Command::new("reg")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .ok();
    }

    fn create_shortcut(exe_path: &Path) {
        let exe = exe_path.to_string_lossy().replace('/', "\\");
        let dir = exe_path.parent().unwrap().to_string_lossy().replace('/', "\\");
        let appdata = std::env::var("APPDATA").unwrap_or_default();

        let shortcut = format!("{appdata}\\Microsoft\\Windows\\Start Menu\\Programs\\MD Reader.lnk");

        let ps = format!(
            "$WS = New-Object -ComObject WScript.Shell; \
             $SC = $WS.CreateShortcut('{shortcut}'); \
             $SC.TargetPath = '{exe}'; \
             $SC.WorkingDirectory = '{dir}'; \
             $SC.IconLocation = '{exe},0'; \
             $SC.Save()",
            shortcut = shortcut,
            exe = exe,
            dir = dir,
        );

        Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .ok();
    }

    fn remove_shortcut() {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let shortcut = format!("{appdata}\\Microsoft\\Windows\\Start Menu\\Programs\\MD Reader.lnk");
        let _ = std::fs::remove_file(shortcut);
    }
}

// ── Linux ──────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use std::path::Path;
    use std::{fs, env};

    pub fn install(exe_path: &Path) {
        let data = data_dir();
        let apps_dir = data.join("applications");
        fs::create_dir_all(&apps_dir).ok();

        let exe = exe_path.to_string_lossy();

        let desktop = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=MD Reader\n\
             Exec={exe} %f\n\
             MimeType=text/markdown;\n\
             Categories=Office;Viewer;\n\
             Terminal=false\n\
             NoDisplay=true\n"
        );

        let desktop_path = apps_dir.join("md-reader.desktop");
        let _ = fs::write(&desktop_path, &desktop);

        // Update mimeapps.list
        let config_dir = data_dir().parent()
            .map(|p| p.join("config"))
            .unwrap_or_else(|| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".into());
                Path::new(&home).join(".config")
            });

        let mime_path = config_dir.join("mimeapps.list");
        let mut content = String::new();
        if mime_path.exists() {
            let _ = fs::read_to_string(&mime_path).map(|s| content = s);
        }

        if !content.contains("md-reader.desktop") {
            let entry = "text/markdown=md-reader.desktop\n";
            if content.contains("[Default Applications]") {
                content = content.replace("[Default Applications]",
                    &format!("[Default Applications]\n{entry}"));
            } else {
                content.push_str(&format!("\n[Default Applications]\n{entry}"));
            }
            let _ = fs::write(&mime_path, &content);
        }

        // xdg-mime also if available
        let _ = std::process::Command::new("xdg-mime")
            .args(["default", "md-reader.desktop", "text/markdown"])
            .status();
    }

    pub fn uninstall() {
        let data = data_dir();
        let desktop_path = data.join("applications").join("md-reader.desktop");
        let _ = fs::remove_file(&desktop_path);

        // Clean mimeapps.list
        let config_dir = data_dir().parent()
            .map(|p| p.join("config"))
            .unwrap_or_else(|| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".into());
                Path::new(&home).join(".config")
            });

        let mime_path = config_dir.join("mimeapps.list");
        if mime_path.exists() {
            if let Ok(content) = fs::read_to_string(&mime_path) {
                let clean = content
                    .lines()
                    .filter(|l| !l.contains("md-reader.desktop"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = fs::write(&mime_path, &clean);
            }
        }
    }

    fn data_dir() -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".into());
                Path::new(&home).join(".local").join("share")
            })
    }
}

// ── macOS ──────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use std::path::{Path, PathBuf};
    use std::{env, fs};

    pub fn install(exe_path: &Path) {
        let bundle_dir = bundle_dir();
        let contents = bundle_dir.join("Contents");
        let macos_dir = contents.join("MacOS");

        fs::create_dir_all(&macos_dir).ok();

        // Info.plist
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>md-reader</string>
    <key>CFBundleIdentifier</key>
    <string>com.mdreader.app</string>
    <key>CFBundleName</key>
    <string>MD Reader</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Markdown File</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>net.daringfireball.markdown</string>
            </array>
        </dict>
    </array>
</dict>
</plist>"#
        );

        let _ = fs::write(contents.join("Info.plist"), &plist);

        // Symlink binary into bundle
        let link = macos_dir.join("md-reader");
        let _ = fs::remove_file(&link);
        let _ = std::os::unix::fs::symlink(exe_path, &link);

        // Register with LaunchServices
        let bundle = bundle_dir.to_string_lossy();
        let _ = std::process::Command::new("lsregister")
            .arg("-f")
            .arg(&*bundle)
            .status();
    }

    pub fn uninstall() {
        let bundle = bundle_dir();

        // Unregister with LaunchServices
        let bundle_str = bundle.to_string_lossy();
        let _ = std::process::Command::new("lsregister")
            .arg("-u")
            .arg(&*bundle_str)
            .status();

        // Remove .app bundle
        if bundle.exists() {
            let _ = fs::remove_dir_all(&bundle);
        }
    }

    fn bundle_dir() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".into());
        Path::new(&home).join("Applications").join("MD Reader.app")
    }
}
