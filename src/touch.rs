//! Read only the touchscreen, never grab it or listen to keyboards.
use evdev::{Device, EventType, KeyCode};
use std::sync::mpsc::{Receiver, sync_channel};
use std::{env, thread, time::Duration};

pub fn listen() -> Receiver<()> {
    let (sender, receiver) = sync_channel(1);
    thread::spawn(move || {
        let mut warned = false;
        loop {
            let device = if let Ok(path) = env::var("ALBERT_EYES_TOUCH_DEVICE") {
                Device::open(path).ok()
            } else {
                evdev::enumerate()
                    .map(|(_, device)| device)
                    .find(|device| device.name() == Some("ADS7846 Touchscreen"))
            };
            let Some(mut device) = device.filter(|device| {
                device
                    .supported_keys()
                    .is_some_and(|keys| keys.contains(KeyCode::BTN_TOUCH))
            }) else {
                if !warned {
                    eprintln!(
                        "Touch unavailable: check ADS7846 connection and /dev/input permissions; retrying."
                    );
                    warned = true;
                }
                thread::sleep(Duration::from_secs(5));
                continue;
            };
            warned = false;
            let mut pressed = false;
            while let Ok(events) = device.fetch_events() {
                for event in events {
                    if is_new_press(
                        &mut pressed,
                        event.event_type(),
                        event.code(),
                        event.value(),
                    ) {
                        // Coalesce taps rather than queueing a long delayed tantrum.
                        let _ = sender.try_send(());
                    }
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
    receiver
}

fn is_new_press(pressed: &mut bool, kind: EventType, code: u16, value: i32) -> bool {
    if kind != EventType::KEY || code != KeyCode::BTN_TOUCH.code() {
        return false;
    }
    match value {
        0 => {
            *pressed = false;
            false
        }
        1 => {
            let new = !*pressed;
            *pressed = true;
            new
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reacts_once_per_contact_not_motion_or_repeat() {
        let mut pressed = false;
        let code = KeyCode::BTN_TOUCH.code();
        assert!(!is_new_press(&mut pressed, EventType::ABSOLUTE, 0, 123));
        assert!(is_new_press(&mut pressed, EventType::KEY, code, 1));
        assert!(!is_new_press(&mut pressed, EventType::KEY, code, 1));
        assert!(!is_new_press(&mut pressed, EventType::KEY, code, 2));
        assert!(!is_new_press(&mut pressed, EventType::KEY, code, 0));
        assert!(is_new_press(&mut pressed, EventType::KEY, code, 1));
    }
}
