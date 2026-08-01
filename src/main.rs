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

    println!("Uptime : --");
    println!("Storage : --");
    println!("Memory : --");
    println!("CPU/HDD Temperature: {} °C/--  ", temperature);
    println!("Copyparty status: -- ");
}
