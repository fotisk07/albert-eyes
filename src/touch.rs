use evdev::{Device, EventSummary, KeyCode};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

const DEVICE_PATH: &str = "/dev/input/by-path/platform-3f204000.spi-cs-1-event";
const RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn listen(sender: Sender<()>) {
    loop {
        let _ = listen_until_error(&sender);
        thread::sleep(RETRY_DELAY);
    }
}

fn listen_until_error(sender: &Sender<()>) -> std::io::Result<()> {
    let mut device = Device::open(DEVICE_PATH)?;

    loop {
        for event in device.fetch_events()? {
            if matches!(
                event.destructure(),
                EventSummary::Key(_, KeyCode::BTN_TOUCH, 1)
            ) {
                sender.send(()).ok();
            }
        }
    }
}
