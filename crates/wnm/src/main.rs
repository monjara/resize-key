mod preferences;
use std::{collections::HashMap, str::FromStr, sync::Arc, thread};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::ns_string;
use wnm_core::frame::{Direction, Edge, move_window, resize};

use crate::preferences::{Operation, Preferences};

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let preferences = Preferences::new();

    let resize_step = preferences.resize_step;
    let move_step = preferences.move_step;

    type Handler = Box<dyn Fn() + Send + Sync + 'static>;
    let mut handlers: HashMap<u32, Handler> = HashMap::new();

    for binding in &preferences.bindings {
        let op: Operation = binding.operation.as_str().into();
        let key = binding.key.as_str();
        let Ok(key) = HotKey::from_str(key) else {
            eprintln!("Invalid key: {}", key);
            continue;
        };

        match op {
            Operation::MoveLeft => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Left, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveRight => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Right, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveUp => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Up, move_step);
                    }) as Handler,
                );
            }
            Operation::MoveDown => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Down, move_step);
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
                        let _ = resize(Edge::Bottom, -resize_step);
                    }) as Handler,
                );
            }
            Operation::ResizeBottomToTop => {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Bottom, resize_step);
                    }) as Handler,
                );
            }
        }
        hotkey_manager.register(key).unwrap();
    }

    let handlers = Arc::new(handlers);

    thread::spawn(move || {
        let rx = GlobalHotKeyEvent::receiver();
        for ev in rx.iter() {
            if ev.state == global_hotkey::HotKeyState::Pressed
                && let Some(h) = handlers.get(&ev.id)
            {
                h();
            }
        }
    });

    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    unsafe {
        let status_bar = NSStatusBar::systemStatusBar();
        let item = status_bar.statusItemWithLength(40.);

        if let Some(button) = item.button(mtm) {
            button.setTitle(ns_string!("hello"));
        }

        let menu = NSMenu::new(mtm);
        let quit = NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!("Quit"),
            Some(sel!(terminate:)),
            ns_string!("q"),
        );
        menu.addItem(&quit);
        item.setMenu(Some(&menu));
    }

    app.run();
}
