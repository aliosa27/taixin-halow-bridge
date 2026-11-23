// Build date (UTC) and git commit for the About page, like the firmware's build time on Status;
// and the Windows .exe icon.
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
mod icon {
    include!("src/icon.rs");
}

fn main() {
    // .exe icon: needs a resource compiler, rc.exe with MSVC (as in CI). The GNU toolchain
    // needs windres, which is often missing, so it gets only the window icon (main.rs).
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        let dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let ico = dir.join("taixin-toolbox.ico");
        std::fs::write(&ico, icon::ico()).unwrap();
        let rc = format!("1 ICON \"{}\"\n", ico.display().to_string().replace('\\', "\\\\"));
        std::fs::write(dir.join("icon.rc"), rc).unwrap();
        embed_resource::compile(dir.join("icon.rc"), embed_resource::NONE);
    }

    let commit = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    println!("cargo:rustc-env=TOOLBOX_COMMIT={commit}");
    println!("cargo:rustc-env=TOOLBOX_BUILD_DATE={}", ymd(secs / 86400));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
}

/// Days since 1970-01-01 to "YYYY-MM-DD" (Howard Hinnant's civil_from_days).
fn ymd(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}
