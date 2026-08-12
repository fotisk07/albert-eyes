use std::fs;

fn main() {
    println!("Hello sir\n\n");

    let temperature: String = match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        Ok(v) => v,
        Err(_e) => String::from("unknown"),
    };

    let temperature: Option<i32> = match temperature.trim().parse::<i32>() {
        Ok(v) => Some(v / 1000),
        Err(_e) => None,
    };

    let temperature = match temperature {
        Some(v) => v.to_string(),
        None => String::from("--"),
    };

    let uptime: Option<String> = match fs::read_to_string("/proc/uptime") {
        Ok(v) => Some(v),
        Err(_e) => None,
    };

    let uptime: Option<String> = match uptime {
        Some(v) => Some(v.split(" ").next().unwrap_or("--").to_string()),
        None => None,
    };

    let uptime: Option<f64> = match uptime {
        Some(v) => match v.parse::<f64>() {
            Ok(n) => Some(n),
            Err(_e) => None,
        },
        None => None,
    };

    let uptime_display = match uptime {
        Some(v) => {
            let days = v / (3600.0 * 24.0);
            let remainder = v % (3600.0 * 24.0);
            let hours = remainder / 3600.0;
            let remainder = remainder % 3600.0;
            let minutes = remainder / 60.0;
            format!("{}d {}h {}m", days as u64, hours as u64, minutes as u64)
        }
        None => String::from("--"),
    };

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

    let storage_display =
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().nth(1).unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let total = parts[0];
        let used = parts[1];
        let percentage = parts[3];

        format!("{} used ({} / {}) ", percentage, used, total)

    } else {String::from("--")};



    println!("Albert's Eyes");
    println!("Uptime           : {}", uptime_display);
    println!("Load Avg         : {}", load_display);
    println!("Storage          : {}", storage_display);
    println!("Memory           : {}", mem_display);
    println!("CPU/HDD Temp     : {} °C / --", temperature);
    println!("Copyparty status : --");
}
