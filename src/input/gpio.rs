use super::ecodes;
use crate::input::{InputDeviceState, InputEvent};
use evdev::raw::input_event;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(PartialEq, Copy, Clone)]
pub enum PhysicalButton {
    LEFT,
    MIDDLE,
    RIGHT,
    POWER,
    WAKEUP,
}

#[derive(PartialEq, Copy, Clone)]
pub enum GPIOEvent {
    Press { button: PhysicalButton },
    Unpress { button: PhysicalButton },
    Unknown,
}

pub struct GPIOState {
    states: [AtomicBool; 5],
}

impl ::std::default::Default for GPIOState {
    fn default() -> Self {
        GPIOState {
            states: [
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
            ],
        }
    }
}

pub fn decode(ev: &input_event, outer_state: &InputDeviceState) -> Option<InputEvent> {
    let state = match outer_state {
        InputDeviceState::GPIOState(ref state_arc) => state_arc,
        _ => unreachable!(),
    };
    match ev._type {
        0 => {
            /* safely ignored. sync event*/
            None
        }
        1 => {
            let (p, before_state) = match ev.code {
                ecodes::KEY_HOME => (
                    PhysicalButton::MIDDLE,
                    state.states[0].fetch_and(ev.value != 0, Ordering::Relaxed),
                ),
                ecodes::KEY_LEFT => (
                    PhysicalButton::LEFT,
                    state.states[1].fetch_and(ev.value != 0, Ordering::Relaxed),
                ),
                ecodes::KEY_RIGHT => (
                    PhysicalButton::RIGHT,
                    state.states[2].fetch_and(ev.value != 0, Ordering::Relaxed),
                ),
                ecodes::KEY_POWER => (
                    PhysicalButton::POWER,
                    state.states[3].fetch_and(ev.value != 0, Ordering::Relaxed),
                ),
                ecodes::KEY_WAKEUP => (
                    PhysicalButton::WAKEUP,
                    state.states[4].fetch_and(ev.value != 0, Ordering::Relaxed),
                ),
                _ => return None,
            };

            // Edge trigger -- debouncing
            let new_state = ev.value != 0;
            if new_state == before_state {
                return None;
            }

            let event = if new_state {
                GPIOEvent::Press { button: p }
            } else {
                GPIOEvent::Unpress { button: p }
            };
            Some(InputEvent::GPIO { event })
        }
        _ => {
            // Shouldn't happen
            error!(
                "Unknown event on PhysicalButtonHandler (type: {0})",
                ev._type
            );
            None
        }
    }
}
