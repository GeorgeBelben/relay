use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use gilrs::{EventType, Gamepad, GamepadId, Gilrs};
use relay_protocol::{ControllerInfo, ControllerType};

fn guess_controller_type(name: &str, vendor_id: Option<u16>) -> ControllerType {
    let lower = name.to_lowercase();

    if lower.contains("xbox") {
        ControllerType::Xbox
    } else if lower.contains("dualsense") || lower.contains("dualshock") {
        ControllerType::Playstation
    } else if lower.contains("nintendo") || lower.contains("switch") || lower.contains("joy-con") {
        ControllerType::Switch
    } else {
        match vendor_id {
            Some(0x054c) => ControllerType::Playstation, // Sony
            Some(0x045e) => ControllerType::Xbox,        // Microsoft
            Some(0x057e) => ControllerType::Switch,      // Nintendo
            _ => ControllerType::Generic,
        }
    }
}

fn to_controller_info(id: GamepadId, gamepad: Gamepad) -> ControllerInfo {
    let index: usize = id.into();

    ControllerInfo {
        index: index as u32,
        name: gamepad.name().to_string(),
        controller_type: guess_controller_type(gamepad.name(), gamepad.vendor_id()),
        vendor_id: gamepad.vendor_id(),
        product_id: gamepad.product_id(),
    }
}

pub type ControllerRegistry = Arc<Mutex<HashMap<u32, ControllerInfo>>>;

pub fn spawn() -> ControllerRegistry {
    let registry: ControllerRegistry = Arc::new(Mutex::new(HashMap::new()));
    let thread_registry = registry.clone();

    std::thread::spawn(move || {
        let mut gilrs = Gilrs::new().expect("failed to initialize gilrs");

        for (id, gamepad) in gilrs.gamepads() {
            let info = to_controller_info(id, gamepad);
            thread_registry.lock().unwrap().insert(info.index, info);
        }

        loop {
            let Some(event) = gilrs.next_event_blocking(None) else {
                continue;
            };

            match event.event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(event.id);
                    let info = to_controller_info(event.id, gamepad);
                    thread_registry.lock().unwrap().insert(info.index, info);
                }
                EventType::Disconnected => {
                    let index: usize = event.id.into();
                    thread_registry.lock().unwrap().remove(&(index as u32));
                }
                _ => {}
            }
        }
    });

    registry
}
