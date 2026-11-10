use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveCleanItem {
    pub id: String,
    pub path: String,
    pub name: String,
    pub category: String, // "system_temp" | "dev_gamer" | "large_file"
    pub size_bytes: u64,
    pub is_dir: bool,
    pub item_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveScanResult {
    pub drive_letter: String,
    pub items: Vec<DriveCleanItem>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveCleanExecutionResult {
    pub success: bool,
    pub deleted_bytes: u64,
    pub deleted_count: u64,
    pub execution_log: Vec<String>,
}

fn measure_dir(path: &Path, max_depth: usize, current_depth: usize) -> (u64, u64) {
    if current_depth > max_depth {
        return (0, 0);
    }
    let mut size = 0u64;
    let mut count = 0u64;
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };

    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                size += meta.len();
                count += 1;
            } else if meta.is_dir() {
                let (sub_size, sub_count) = measure_dir(&entry.path(), max_depth, current_depth + 1);
                size += sub_size;
                count += sub_count;
            }
        }
    }
    (size, count)
}

/// Scan a specific drive (e.g. "C:", "D:", "E:") for:
/// 1. System Temp & logs
/// 2. Dev & Gamer Caches (`node_modules`, `__pycache__`, `.gradle`, DirectX shader cache)
/// 3. Large files > 1GB
pub fn scan_drive_for_cleanup(drive: &str) -> DriveScanResult {
    let drive_prefix = drive.trim().trim_end_matches(['\\', '/']).to_uppercase();
    let root = format!("{}\\", drive_prefix);
    let mut items = Vec::new();
    let mut total_bytes = 0u64;

    // 1. System Temp & Windows Logs (If scanning C: or if Temp exists on drive)
    if drive_prefix == "C:" {
        let temp_dirs = [
            std::env::var("TEMP").unwrap_or_default(),
            "C:\\Windows\\Temp".to_string(),
            "C:\\Windows\\Logs".to_string(),
        ];
        for t in &temp_dirs {
            let p = Path::new(t);
            if p.exists() && p.is_dir() {
                let (sz, count) = measure_dir(p, 3, 0);
                if sz > 1024 * 1024 {
                    // > 1MB
                    items.push(DriveCleanItem {
                        id: format!("temp_{}", items.len()),
                        path: t.clone(),
                        name: format!("Temp / Log Cache ({})", p.file_name().and_then(|n| n.to_str()).unwrap_or("Temp")),
                        category: "system_temp".to_string(),
                        size_bytes: sz,
                        is_dir: true,
                        item_count: count,
                    });
                    total_bytes += sz;
                }
            }
        }

        // DirectX Shader Cache
        let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        if !local_appdata.is_empty() {
            let shader_paths = [
                format!("{}\\D3DSCache", local_appdata),
                format!("{}\\NVIDIA\\DXCache", local_appdata),
                format!("{}\\AMD\\DxCache", local_appdata),
            ];
            for sp in &shader_paths {
                let p = Path::new(sp);
                if p.exists() && p.is_dir() {
                    let (sz, count) = measure_dir(p, 3, 0);
                    if sz > 512 * 1024 {
                        items.push(DriveCleanItem {
                            id: format!("shader_{}", items.len()),
                            path: sp.clone(),
                            name: format!("Shader Cache ({})", p.file_name().and_then(|n| n.to_str()).unwrap_or("Cache")),
                            category: "dev_gamer".to_string(),
                            size_bytes: sz,
                            is_dir: true,
                            item_count: count,
                        });
                        total_bytes += sz;
                    }
                }
            }
        }
    }

    // 2. Scan for Dev Caches (`node_modules`, `__pycache__`, `.gradle`) and Large Files (>1GB)
    // Scan up to depth 4 from root or known common directories
    let common_subdirs = [
        "",
        "Users",
        "Projects",
        "Code",
        "Work",
        "Games",
        "Dev",
        "Repositories",
    ];

    for sub in &common_subdirs {
        let base_path = if sub.is_empty() {
            PathBuf::from(&root)
        } else {
            PathBuf::from(format!("{}\\{}", root.trim_end_matches('\\'), sub))
        };

        if !base_path.exists() {
            continue;
        }

        scan_recursive_candidates(&base_path, 3, 0, &mut items, &mut total_bytes);
        if items.len() >= 40 {
            break;
        }
    }

    DriveScanResult {
        drive_letter: drive_prefix,
        items,
        total_bytes,
    }
}

fn scan_recursive_candidates(
    dir: &Path,
    max_depth: usize,
    depth: usize,
    items: &mut Vec<DriveCleanItem>,
    total_bytes: &mut u64,
) {
    if depth > max_depth || items.len() >= 40 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy();

        // Skip Windows / Program Files system internals for speed & safety
        if name.starts_with('$') || name == "Windows" || name == "System Volume Information" || name == "Recovery" {
            continue;
        }

        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                // Dev caches
                if name == "node_modules" || name == "__pycache__" || name == ".gradle" || name == "target" && path.join("CACHEDIR.TAG").exists() {
                    let (sz, count) = measure_dir(&path, 3, 0);
                    if sz > 10 * 1024 * 1024 {
                        // > 10MB
                        items.push(DriveCleanItem {
                            id: format!("dev_{}", items.len()),
                            path: path.to_string_lossy().to_string(),
                            name: format!("{} in {}", name, dir.file_name().and_then(|n| n.to_str()).unwrap_or("")),
                            category: "dev_gamer".to_string(),
                            size_bytes: sz,
                            is_dir: true,
                            item_count: count,
                        });
                        *total_bytes += sz;
                    }
                    continue;
                }

                scan_recursive_candidates(&path, max_depth, depth + 1, items, total_bytes);
            } else if meta.is_file() {
                // Large files (> 1GB = 1,073,741,824 bytes)
                if meta.len() >= 1024 * 1024 * 1024 {
                    items.push(DriveCleanItem {
                        id: format!("large_{}", items.len()),
                        path: path.to_string_lossy().to_string(),
                        name: format!("Large File: {}", name),
                        category: "large_file".to_string(),
                        size_bytes: meta.len(),
                        is_dir: false,
                        item_count: 1,
                    });
                    *total_bytes += meta.len();
                }
            }
        }
    }
}

/// Delete selected files or directories with real-time log tracking.
pub fn delete_selected_drive_items(paths: Vec<String>) -> DriveCleanExecutionResult {
    let mut deleted_bytes = 0u64;
    let mut deleted_count = 0u64;
    let mut log = Vec::new();

    for p_str in paths {
        let p = Path::new(&p_str);
        if !p.exists() {
            log.push(format!("[SKIP] Not found: {p_str}"));
            continue;
        }

        if let Ok(meta) = p.metadata() {
            if meta.is_dir() {
                let (sz, cnt) = measure_dir(p, 4, 0);
                match fs::remove_dir_all(p) {
                    Ok(_) => {
                        deleted_bytes += sz;
                        deleted_count += cnt;
                        log.push(format!("[DELETED DIR] {p_str} ({cnt} files, freed {sz} bytes)"));
                    }
                    Err(e) => {
                        log.push(format!("[ERROR DIR] {p_str}: {e}"));
                    }
                }
            } else {
                let sz = meta.len();
                match fs::remove_file(p) {
                    Ok(_) => {
                        deleted_bytes += sz;
                        deleted_count += 1;
                        log.push(format!("[DELETED FILE] {p_str} (freed {sz} bytes)"));
                    }
                    Err(e) => {
                        log.push(format!("[ERROR FILE] {p_str}: {e}"));
                    }
                }
            }
        }
    }

    DriveCleanExecutionResult {
        success: true,
        deleted_bytes,
        deleted_count,
        execution_log: log,
    }
}

