use std::process::Command;

fn main() {
    // Compile GResource
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Check if glib-compile-resources is available
    let status = Command::new("glib-compile-resources")
        .args([
            "--sourcedir=data",
            &format!(
                "--target={}/com.chrisdaggas.network-manager.gresource",
                out_dir
            ),
            "data/com.chrisdaggas.network-manager.gresource.xml",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rerun-if-changed=data/com.chrisdaggas.network-manager.gresource.xml");
            println!("cargo:rerun-if-changed=data/style.css");
            println!("cargo:rerun-if-changed=data/icons/");
            println!("cargo:rerun-if-changed=data/images/");
        }
        _ => {
            if std::env::var("ALLOW_MISSING_GRESOURCE").is_ok() {
                eprintln!("Warning: glib-compile-resources not found or failed. Continuing because ALLOW_MISSING_GRESOURCE is set.");
            } else {
                panic!("glib-compile-resources failed. Install libglib2.0-dev or set ALLOW_MISSING_GRESOURCE=1 to skip.");
            }
        }
    }

    // Tell cargo to rerun if any source files change
    println!("cargo:rerun-if-changed=src/");

    // Compile gettext catalogs so `cargo run` uses the same translations as
    // an installed build. Packaging installs target/locale.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let locale_out = std::path::Path::new(&manifest_dir).join("target/locale");
    let po_dir = std::path::Path::new(&manifest_dir).join("po");

    println!("cargo:rerun-if-changed=po/");
    match Command::new("msgfmt").arg("--version").output() {
        Ok(_) => {
            let _ = std::fs::remove_dir_all(&locale_out);
            if let Ok(entries) = std::fs::read_dir(&po_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) != Some("po") {
                        continue;
                    }
                    let Some(lang) = path.file_stem().and_then(|name| name.to_str()) else {
                        continue;
                    };
                    let lang_dir = locale_out.join(lang).join("LC_MESSAGES");
                    let _ = std::fs::create_dir_all(&lang_dir);
                    let mo_path = lang_dir.join("network-manager.mo");
                    let status = Command::new("msgfmt")
                        .arg("-o")
                        .arg(&mo_path)
                        .arg(&path)
                        .status();
                    if let Ok(status) = status {
                        if !status.success() {
                            println!("cargo:warning=msgfmt failed for {:?}", path);
                        }
                    }
                }
            }
        }
        Err(_) => {
            println!("cargo:warning=msgfmt not found; translations will not be available in development builds");
        }
    }
}
