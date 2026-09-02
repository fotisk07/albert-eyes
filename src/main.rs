mod animation;
mod collect;
mod render;
mod status;
use std::io::Write;
use std::thread;
use std::time;

const RENDER_UPDATE_MSECS: u64 = 100;
const STATUS_UPDATE_MSECS: u64 = 1000;

fn main() {
    let mut status = collect::collect_status();
    let mut last_collection = time::Instant::now();
    let mut animation = animation::Animator::new();

    print!("\x1B[2J");

    loop {
        if last_collection.elapsed() >= time::Duration::from_millis(STATUS_UPDATE_MSECS) {
            status = collect::collect_status();
            last_collection = time::Instant::now();
        }
        animation.update();
        print!("\x1B[H{}", render::render(&status, animation.pose()));
        let _ = std::io::stdout().flush();
        thread::sleep(time::Duration::from_millis(RENDER_UPDATE_MSECS));
    }
}
