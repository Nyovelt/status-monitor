use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use sysinfo::{Disks, System};
use tracing::{debug, warn};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub cpu_usage: f64,
    pub ram_usage: f64,
    pub disk_usage: f64,
    pub inode_usage: f64,
    pub docker_sz: Option<i64>,
    pub gpu_usage: Option<f64>,
    pub timestamp: String,
}

pub struct MetricCollector {
    system: System,
    disks: Disks,
    docker_path: String,
    cached_docker_size: Arc<AtomicU64>,
    nvml: Option<nvml_wrapper::Nvml>,
}

impl MetricCollector {
    pub fn new(docker_path: String) -> Self {
        let nvml = nvml_wrapper::Nvml::init().ok();
        if nvml.is_some() {
            debug!("NVIDIA NVML initialized successfully");
        } else {
            debug!("NVIDIA NVML not available (no GPU or driver not installed)");
        }

        Self {
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            docker_path,
            cached_docker_size: Arc::new(AtomicU64::new(0)),
            nvml,
        }
    }

    /// Collect fast metrics (CPU, RAM, GPU, inodes) - called every 1 second
    pub fn collect_fast(&mut self) -> Metric {
        // Refresh CPU and memory
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        let cpu_usage = self.system.global_cpu_usage() as f64;
        let ram_usage = self.calculate_ram_usage();
        let (disk_usage, inode_usage) = self.calculate_disk_usage();
        let gpu_usage = self.collect_gpu_usage();

        // Use cached docker size
        let docker_sz = {
            let cached = self.cached_docker_size.load(Ordering::Relaxed);
            if cached > 0 {
                Some(cached as i64)
            } else {
                None
            }
        };

        Metric {
            cpu_usage,
            ram_usage,
            disk_usage,
            inode_usage,
            docker_sz,
            gpu_usage,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Calculate Docker directory size - called every 5 minutes
    pub fn update_docker_size(&self) {
        let path = Path::new(&self.docker_path);
        if !path.exists() {
            debug!("Docker path {} does not exist", self.docker_path);
            return;
        }

        let size = calculate_dir_size(path);
        self.cached_docker_size.store(size, Ordering::Relaxed);
        debug!("Updated Docker size: {} bytes", size);
    }

    pub fn get_cached_docker_size(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.cached_docker_size)
    }

    fn calculate_ram_usage(&self) -> f64 {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        if total == 0 {
            return 0.0;
        }
        (used as f64 / total as f64) * 100.0
    }

    fn calculate_disk_usage(&mut self) -> (f64, f64) {
        self.disks.refresh();

        // Find root disk
        let root_disk = self.disks.iter().find(|d| d.mount_point() == Path::new("/"));

        if let Some(disk) = root_disk {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let disk_usage = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            // Calculate inode usage using statfs
            let inode_usage = calculate_inode_usage("/");

            (disk_usage, inode_usage)
        } else {
            warn!("Root disk not found");
            (0.0, 0.0)
        }
    }

    fn collect_gpu_usage(&self) -> Option<f64> {
        let nvml = self.nvml.as_ref()?;

        // Try to get the first GPU
        let device = nvml.device_by_index(0).ok()?;
        let utilization = device.utilization_rates().ok()?;

        Some(utilization.gpu as f64)
    }
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(target_os = "linux")]
fn calculate_inode_usage(path: &str) -> f64 {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = match CString::new(path) {
        Ok(p) => p,
        Err(_) => return 0.0,
    };

    unsafe {
        let mut stat: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
        if libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let total_inodes = stat.f_files;
            let free_inodes = stat.f_ffree;
            if total_inodes > 0 {
                let used_inodes = total_inodes - free_inodes;
                return (used_inodes as f64 / total_inodes as f64) * 100.0;
            }
        }
    }
    0.0
}

#[cfg(not(target_os = "linux"))]
fn calculate_inode_usage(_path: &str) -> f64 {
    0.0
}
