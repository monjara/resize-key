use std::{collections::HashMap, sync::Arc, thread};

use core::frame::{Direction, Edge, move_window_nswindow_style, resize};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};

use crate::preferences::{Operation, Preferences};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    handlers: Arc<HashMap<u32, Box<dyn Fn() + Send + Sync + 'static>>>,
}

impl HotkeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        let handlers = Arc::new(HashMap::new());

        Ok(Self { manager, handlers })
    }

    pub fn register_hotkeys(
        &mut self,
        preferences: &Preferences,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resize_step = preferences.resize_step;
        let move_step = preferences.move_step;

        type Handler = Box<dyn Fn() + Send + Sync + 'static>;
        let mut handlers: HashMap<u32, Handler> = HashMap::new();

        for binding in &preferences.bindings {
            let op: Operation = binding.operation.as_str().into();
            let key = binding.key.as_str();
            let Ok(hotkey): Result<HotKey, _> = key.parse() else {
                continue;
            };

            let handler: Handler = match op {
                Operation::MoveLeft => Box::new(move || {
                    let _ = move_window_nswindow_style(&Direction::Left, move_step);
                }),
                Operation::MoveRight => Box::new(move || {
                    let _ = move_window_nswindow_style(&Direction::Right, move_step);
                }),
                Operation::MoveUp => Box::new(move || {
                    let _ = move_window_nswindow_style(&Direction::Up, move_step);
                }),
                Operation::MoveDown => Box::new(move || {
                    let _ = move_window_nswindow_style(&Direction::Down, move_step);
                }),
                Operation::ResizeLeftToLeft => Box::new(move || {
                    let _ = resize(Edge::Left, -resize_step);
                }),
                Operation::ResizeLeftToRight => Box::new(move || {
                    let _ = resize(Edge::Left, resize_step);
                }),
                Operation::ResizeRightToLeft => Box::new(move || {
                    let _ = resize(Edge::Right, -resize_step);
                }),
                Operation::ResizeRightToRight => Box::new(move || {
                    let _ = resize(Edge::Right, resize_step);
                }),
                Operation::ResizeTopToTop => Box::new(move || {
                    let _ = resize(Edge::Top, resize_step);
                }),
                Operation::ResizeTopToBottom => Box::new(move || {
                    let _ = resize(Edge::Top, -resize_step);
                }),
                Operation::ResizeBottomToBottom => Box::new(move || {
                    let _ = resize(Edge::Bottom, resize_step);
                }),
                Operation::ResizeBottomToTop => Box::new(move || {
                    let _ = resize(Edge::Bottom, -resize_step);
                }),
            };

            handlers.insert(hotkey.id(), handler);
            self.manager.register(hotkey)?;
        }

        self.handlers = Arc::new(handlers);
        Ok(())
    }

    pub fn start_monitoring(&self) {
        let handlers = Arc::clone(&self.handlers);

        thread::spawn(move || {
            let rx = GlobalHotKeyEvent::receiver();
            for event in rx {
                if event.state == global_hotkey::HotKeyState::Pressed {
                    if let Some(handler) = handlers.get(&event.id) {
                        handler();
                    }
                }
            }
        });
    }
}

