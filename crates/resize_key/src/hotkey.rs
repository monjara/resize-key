use core::frame::{Direction, Edge, move_window_nswindow_style, resize};
use std::{collections::HashMap, str::FromStr, sync::Arc, thread};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};

use crate::preferences::{Operation, Preferences};

type Handler = Box<dyn Fn() + Send + Sync + 'static>;

pub(crate) fn register_hotkeys(preferences: &Preferences) -> anyhow::Result<()> {
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let mut handlers: HashMap<u32, Handler> = HashMap::new();

    let resize_step = preferences.resize_step;
    let move_step = preferences.move_step;

    for binding in &preferences.bindings {
        let op: Operation = binding.operation.as_str().into();
        let key = binding.key.as_str();
        let Ok(key) = HotKey::from_str(key) else {
            continue;
        };

        match op {
            Operation::MoveLeft => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window_nswindow_style(&Direction::Left, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveRight => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window_nswindow_style(&Direction::Right, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveUp => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window_nswindow_style(&Direction::Up, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveDown => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window_nswindow_style(&Direction::Down, move_step);
                    }) as Handler,
                );
            }
            Operation::ResizeLeftToLeft => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Left, -resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeLeftToRight => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Left, resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeRightToLeft => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Right, -resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeRightToRight => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Right, resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeTopToTop => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Top, resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeTopToBottom => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Top, -resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeBottomToBottom => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Bottom, resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeBottomToTop => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Bottom, -resize_step);
                    }) as Handler,
                );
            }
        }
        hotkey_manager.register(key).unwrap();
    }

    let handlers = Arc::new(handlers);

    thread::spawn(move || {
        let rx = GlobalHotKeyEvent::receiver();
        for ev in rx {
            if ev.state == global_hotkey::HotKeyState::Pressed
                && let Some(h) = handlers.get(&ev.id)
            {
                h();
            }
        }
    });

    Ok(())
}
