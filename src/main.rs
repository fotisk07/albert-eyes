mod animation;
mod collect;
mod render;
mod status;
use render::DisplayMode;
use std::env;
use std::io::Write;
use std::thread;
use std::time;

const RENDER_UPDATE_MSECS: u64 = 100;
const STATUS_UPDATE_MSECS: u64 = 1000;

fn mode_from_name(name: &str) -> Result<Option<DisplayMode>, String> {
    match name.to_ascii_lowercase().as_str() {
        "auto" => Ok(None),
        "pc" => Ok(Some(DisplayMode::Pc)),
        "screen" => Ok(Some(DisplayMode::Screen)),
        _ => Err(format!(
            "unknown mode '{name}'; expected auto, pc, or screen"
        )),
    }
}

fn display_mode() -> Result<DisplayMode, String> {
    let mut requested = env::var("ALBERT_EYES_MODE").unwrap_or_else(|_| "auto".into());
    let mut args = env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--mode" => {
                requested = args
                    .next()
                    .ok_or_else(|| "--mode requires auto, pc, or screen".to_string())?;
            }
            "-h" | "--help" => {
                println!("Usage: albert-eyes [--mode auto|pc|screen]");
                println!(
                    "\nALBERT_EYES_MODE can also set the mode. Auto uses screen mode on a Linux console."
                );
                std::process::exit(0);
            }
            _ if argument.starts_with("--mode=") => {
                requested = argument["--mode=".len()..].to_string();
            }
            _ => return Err(format!("unknown argument '{argument}'")),
        }
    }

    Ok(mode_from_name(&requested)?.unwrap_or_else(|| {
        if env::var("TERM").as_deref() == Ok("linux") {
            DisplayMode::Screen
        } else {
            DisplayMode::Pc
        }
    }))
}

fn main() {
    let mode = display_mode().unwrap_or_else(|error| {
        eprintln!("albert-eyes: {error}");
        std::process::exit(2);
    });
    let mut status = collect::collect_status();
    let mut last_collection = time::Instant::now();
    let mut animation = animation::Animator::new();

    print!("\x1B[2J");

    loop {
        if last_collection.elapsed() >= time::Duration::from_millis(STATUS_UPDATE_MSECS) {
            status = collect::collect_status();
            last_collection = time::Instant::now();
        }
        animation.update(&status);
        print!("\x1B[H{}", render::render(&status, animation.pose(), mode));
        let _ = std::io::stdout().flush();
        thread::sleep(time::Duration::from_millis(RENDER_UPDATE_MSECS));
    }
}
