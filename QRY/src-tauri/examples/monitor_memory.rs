//! Native macOS monitor-query soak test. No input monitoring or database access.

#[cfg(target_os = "macos")]
#[path = "../src/native_scope.rs"]
mod native_scope;

#[cfg(target_os = "macos")]
fn footprint_bytes() -> Result<u64, String> {
    let output = std::process::Command::new("/usr/bin/footprint")
        .args([&std::process::id().to_string(), "-f", "bytes"])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("footprint could not inspect the probe process".into());
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("phys_footprint:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .ok_or_else(|| "footprint did not report physical memory".into())
}

#[cfg(target_os = "macos")]
fn soak(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    let refresh = || {
        native_scope::run(|| -> tauri::Result<()> {
            // Exercise the overlay's app-level and window-level native queries
            // from a Rust worker, including a focused-display lookup.
            let monitors = app.available_monitors()?;
            let _ = app.primary_monitor()?;
            if let Some(monitor) = monitors.first() {
                let point = monitor.position();
                let _ = app.monitor_from_point(f64::from(point.x), f64::from(point.y))?;
            }
            let _ = window.current_monitor()?;
            let _ = window.scale_factor()?;
            let _ = window.inner_size()?;
            let _ = window.outer_position()?;
            Ok(())
        })
        .map_err(|error| error.to_string())
    };

    for _ in 0..1_000 {
        refresh()?;
    }
    let baseline = footprint_bytes()?;
    println!("warmup=1000 footprint_bytes={baseline}");
    let mut peak = baseline;
    for batch in 1..=4 {
        for _ in 0..5_000 {
            refresh()?;
        }
        let footprint = footprint_bytes()?;
        peak = peak.max(footprint);
        println!("refreshes={} footprint_bytes={footprint}", batch * 5_000);
    }
    let growth = peak.saturating_sub(baseline);
    // Allow allocator/runtime warmup noise, while rejecting sustained native
    // allocation growth across repeated refreshes.
    if growth > 16 * 1024 * 1024 {
        return Err(format!(
            "native footprint grew by {growth} bytes (limit: 16 MiB)"
        ));
    }
    println!("PASS refreshes=20000 peak_growth_bytes={growth}");
    Ok(())
}

#[cfg(target_os = "macos")]
fn main() {
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    context
        .config_mut()
        .identifier
        .push_str(".monitor-memory-check");
    tauri::Builder::default()
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "memory-check",
                tauri::WebviewUrl::External(tauri::Url::parse("about:blank")?),
            )
            .visible(false)
            .build()?;
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("qry-monitor-memory-check".into())
                .spawn(move || {
                    let result = soak(&handle, &window);
                    if let Err(error) = &result {
                        eprintln!("FAIL: {error}");
                    }
                    handle.exit(i32::from(result.is_err()));
                })?;
            Ok(())
        })
        .run(context)
        .expect("native memory check failed to start");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This native monitor memory check requires macOS.");
    std::process::exit(1);
}
