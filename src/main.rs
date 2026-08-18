use chrono::{DateTime, Utc};
use std::fs;
use std::time::Duration;

struct StatusSnapshot {
    temperature: Option<i32>,
    uptime: Option<Duration>,
    load: String,
    memory: String,
    storage: String,
    copyparty: String,
    backup: String,

    collected_at: DateTime<Utc>,
}

fn render(snapshot: &StatusSnapshot) {
    let temperature = match snapshot.temperature {
        Some(v) => v.to_string(),
        None => String::from("--"),
    };

    let uptime = match snapshot.uptime {
        Some(v) => {
            let sec = v.as_secs();
            let days = sec / (3600 * 24);
            let remainder = sec % (3600 * 24);
            let hours = remainder / 3600;
            let remainder = remainder % 3600;
            let minutes = remainder / 60;
            format!("{}d {}h {}m", days, hours, minutes)
        }
        None => String::from("--"),
    };

    println!("Albert's Eyes");
    println!("CPU/HDD Temp     : {} °C / --", temperature);
    println!("Uptime           : {}", uptime);
    println!("Load Avg         : {}", snapshot.load);
    println!("Storage          : {}", snapshot.storage);
    println!("Memory           : {}", snapshot.memory);
    println!("\n");
    println!("Copyparty status : {}", snapshot.copyparty);
    println!("Last Backup      : {}", snapshot.backup);

    println!("Collected at {}", snapshot.collected_at)
}

fn collect_temperature() -> Option<i32> {
    let temperature: String = match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        Ok(v) => v,
        Err(_e) => return None,
    };

    temperature
        .trim()
        .parse::<i32>()
        .ok()
        .map(|value| value / 1000)
}

fn collect_uptime() -> Option<Duration> {
    let uptime: String = fs::read_to_string("/proc/uptime").ok()?;
    let uptime = uptime.split_whitespace().next()?;
    let uptime: f64 = uptime.parse::<f64>().ok()?;

    Duration::try_from_secs_f64(uptime).ok()
}

fn collect_status() -> StatusSnapshot {
    let load_avg: Option<String> = match fs::read_to_string("/proc/loadavg") {
        Ok(v) => Some(v),
        Err(_e) => None,
    };
    let load_display: String = match load_avg {
        Some(v) => {
            let mut fields = v.split(" ");
            let one_min = fields.next().unwrap_or("--").to_string();
            let five_min = fields.next().unwrap_or("--").to_string();
            let fifteen_min = fields.next().unwrap_or("--").to_string();

            format!("1m {} · 5m {} · 15m {}", one_min, five_min, fifteen_min)
        }
        None => String::from("--"),
    };

    let meminfo: Option<String> = match fs::read_to_string("/proc/meminfo") {
        Ok(v) => Some(v),
        Err(_e) => None,
    };

    let mem_display = match meminfo {
        Some(v) => {
            let mem_total = v
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|n| n.parse::<f64>().ok());

            let mem_available = v
                .lines()
                .find(|line| line.starts_with("MemAvailable:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|n| n.parse::<f64>().ok());

            match (mem_total, mem_available) {
                (Some(total), Some(available)) => {
                    let used_mib = (total - available) / 1024.0;
                    let total_mib = total / 1024.0;
                    format!("{} MiB / {} MiB", used_mib as u64, total_mib as u64)
                }
                _ => String::from("--"),
            }
        }
        None => String::from("--"),
    };

    use std::process::Command;

    let output = Command::new("df")
        .args([
            "--output=size,used,avail,pcent,target",
            "-h",
            "/srv/storage",
        ])
        .output()
        .expect("Failed to run df");

    let storage_display = if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().nth(1).unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let total = parts[0];
        let used = parts[1];
        let percentage = parts[3];

        format!("{} used ({} / {}) ", percentage, used, total)
    } else {
        String::from("--")
    };

    let output = Command::new("systemctl")
        .args(["--user", "is-active", "copyparty"])
        .output();

    let copyparty_display = match output {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout);
            status.trim().to_string()
        }
        Err(_) => "unknown".to_string(),
    };

    let restic = Command::new("restic")
        .args([
            "--password-file",
            "/home/fotis/.config/albert-eyes/restic/restic-password",
            "-r",
            "/srv/storage/backups/dell-pc/",
            "snapshots",
            "--latest",
            "1",
            "--json",
        ])
        .output();

    let restic_display = match restic {
        Ok(v) => {
            if v.status.success() {
                let restic_json = String::from_utf8_lossy(&v.stdout).to_string();

                let parsed: serde_json::Value = serde_json::from_str(&restic_json).unwrap();

                let time_str = parsed[0]["time"].as_str().unwrap_or("Unknown").to_string();

                let snapshot_time = DateTime::parse_from_rfc3339(&time_str);

                match snapshot_time {
                    Ok(v) => {
                        let now = Utc::now();

                        let age = now - v.with_timezone(&Utc);

                        if age.num_hours() > 48 {
                            format!("Stale ({} hours ago)", age.num_hours())
                        } else {
                            format!("Current ({} hours ago)", age.num_hours())
                        }
                    }

                    Err(_) => String::from("Unavailable"),
                }
            } else {
                String::from("Error")
            }
        }

        Err(_) => String::from("Error"),
    };

    let snapshot = StatusSnapshot {
        temperature: collect_temperature(),
        uptime: collect_uptime(),
        load: load_display,
        memory: mem_display,
        storage: storage_display,
        copyparty: copyparty_display,
        backup: restic_display,
        collected_at: Utc::now(),
    };

    snapshot
}

fn main() {
    let status = collect_status();
    render(&status);
}
