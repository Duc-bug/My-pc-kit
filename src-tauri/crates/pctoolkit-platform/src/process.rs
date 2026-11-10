use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use sysinfo::{ProcessesToUpdate, System};

use crate::{PlatformError, PlatformResult};

/// Windows `CREATE_NO_WINDOW` — hide console for background tooling (powershell, net, etc.).
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn hide_console(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Returns a Command configured to run Windows PowerShell with full path resolution.
pub fn powershell_cmd() -> Command {
    let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let sys_ps = format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", sys_root);
    if std::path::Path::new(&sys_ps).exists() {
        return Command::new(sys_ps);
    }
    if let Ok(output) = Command::new("pwsh.exe").arg("-v").output() {
        if output.status.success() {
            return Command::new("pwsh.exe");
        }
    }
    Command::new("powershell.exe")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcessGroup {
    pub image_name: String,
    pub process_count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCloseTargetResult {
    pub image_name: String,
    pub status: String,
    pub remaining_count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCloseBatchResult {
    pub targets: Vec<ProcessCloseTargetResult>,
}

/// Image names that Deep Cleanup is allowed to close (case-insensitive).
const CLOSE_ALLOWLIST: &[&str] = &[
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "brave.exe",
    "opera.exe",
    "opera_gx.exe",
    "vivaldi.exe",
    "chromium.exe",
    "discord.exe",
    "telegram.exe",
    "whatsapp.exe",
    "whatsapp.desktop.exe",
    "whatsapp.root.exe",
    "steam.exe",
    "steamwebhelper.exe",
    "ms-teams.exe",
    "teams.exe",
    "spotify.exe",
    "slack.exe",
    "notion.exe",
    "docker desktop.exe",
    "code.exe",
    "cursor.exe",
    "vscodium.exe",
    "windsurf.exe",
];

fn normalize_image(name: &str) -> String {
    let trimmed = name.trim();
    let with_exe = if trimmed.to_ascii_lowercase().ends_with(".exe") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.exe")
    };
    with_exe.to_ascii_lowercase()
}

fn is_allowed(image: &str) -> bool {
    CLOSE_ALLOWLIST
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(image))
}

fn refresh_system() -> System {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys
}

fn count_by_image(sys: &System, images: &HashSet<String>) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for process in sys.processes().values() {
        let name = normalize_image(&process.name().to_string_lossy());
        if images.contains(&name) {
            *counts.entry(name).or_default() += 1;
        }
    }
    counts
}

/// Probe which of the requested image names are currently running.
pub fn probe_running_processes(names: &[String]) -> PlatformResult<Vec<RunningProcessGroup>> {
    let images: HashSet<String> = names
        .iter()
        .map(|n| normalize_image(n))
        .filter(|n| is_allowed(n))
        .collect();
    if images.is_empty() {
        return Ok(Vec::new());
    }
    let sys = refresh_system();
    let counts = count_by_image(&sys, &images);
    let mut groups: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(image_name, process_count)| RunningProcessGroup {
            image_name,
            process_count,
        })
        .collect();
    groups.sort_by(|a, b| a.image_name.cmp(&b.image_name));
    Ok(groups)
}

fn taskkill(image: &str, force: bool) -> PlatformResult<()> {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/IM", image]);
    if force {
        cmd.arg("/F");
    }
    hide_console(&mut cmd);
    let status = cmd
        .status()
        .map_err(|e| PlatformError::OperationFailed(e.to_string()))?;
    // taskkill returns non-zero when the process is already gone — treat as ok.
    if status.success() || !force {
        return Ok(());
    }
    Ok(())
}

fn kill_via_sysinfo(sys: &System, image: &str) {
    let target = normalize_image(image);
    for process in sys.processes().values() {
        let name = normalize_image(&process.name().to_string_lossy());
        if name == target {
            let _ = process.kill();
        }
    }
}

/// Close matching processes. `force=false` tries a soft taskkill first; remaining
/// processes are reported so the UI can offer a force retry.
pub fn close_processes(names: &[String], force: bool) -> PlatformResult<ProcessCloseBatchResult> {
    let images: Vec<String> = names
        .iter()
        .map(|n| normalize_image(n))
        .filter(|n| is_allowed(n))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if images.is_empty() {
        return Ok(ProcessCloseBatchResult { targets: vec![] });
    }

    for image in &images {
        let _ = taskkill(image, force);
        if force {
            let sys = refresh_system();
            kill_via_sysinfo(&sys, image);
        }
    }

    // Give Windows a moment to tear down handles before re-probing.
    thread::sleep(Duration::from_millis(if force { 400 } else { 700 }));
    let sys = refresh_system();
    let image_set: HashSet<String> = images.iter().cloned().collect();
    let remaining = count_by_image(&sys, &image_set);

    let mut targets = Vec::new();
    for image in images {
        let left = *remaining.get(&image).unwrap_or(&0);
        let status = if left == 0 {
            "closed"
        } else if force {
            "failed"
        } else {
            "remaining"
        };
        targets.push(ProcessCloseTargetResult {
            image_name: image,
            status: status.into(),
            remaining_count: left,
        });
    }
    targets.sort_by(|a, b| a.image_name.cmp(&b.image_name));
    Ok(ProcessCloseBatchResult { targets })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_mb: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionLog {
    pub success: bool,
    pub command: String,
    pub output: String,
}

/// Retrieve top active processes sorted by memory usage or CPU load.
pub fn get_top_resource_processes(limit: usize) -> Vec<TopProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    // Sleep briefly to calculate CPU delta
    thread::sleep(Duration::from_millis(150));
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut list = Vec::new();
    for (pid, proc) in sys.processes() {
        let name = proc.name().to_string_lossy().to_string();
        if name.is_empty() || name == "System Idle Process" || name == "System" {
            continue;
        }
        let mem = proc.memory();
        let cpu = proc.cpu_usage();
        list.push(TopProcessInfo {
            pid: pid.as_u32(),
            name,
            cpu_percent: (cpu * 10.0).round() / 10.0,
            memory_bytes: mem,
            memory_mb: (mem as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0,
        });
    }

    // Sort descending by memory usage first, then CPU
    list.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    list.truncate(limit);
    list
}

/// Terminate a process by PID with real-time execution log output.
pub fn terminate_process_by_pid(pid: u32) -> ProcessActionLog {
    let cmd_str = format!("taskkill /F /PID {pid}");
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/PID", &pid.to_string()]);
    hide_console(&mut cmd);

    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let combined = if stderr.is_empty() { stdout } else { format!("{stdout}\n{stderr}") };
            ProcessActionLog {
                success: out.status.success(),
                command: cmd_str,
                output: combined,
            }
        }
        Err(e) => ProcessActionLog {
            success: false,
            command: cmd_str,
            output: format!("Execution failed: {e}"),
        },
    }
}
