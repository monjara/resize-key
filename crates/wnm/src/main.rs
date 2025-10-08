mod preferences;
use std::{collections::HashMap, str::FromStr, sync::Arc, thread};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::ns_string;
use wnm_core::frame::{Direction, Edge, move_window, resize};

use crate::preferences::Preferences;

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
        let op = binding.operation.as_str();
        let key = binding.key.as_str();

        if op == "move_left" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Left, move_step);
                    }) as Handler,
                );
                hotkey_manager.register(key).unwrap();
            }
        } else if op == "move_right" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Right, move_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "move_up" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Up, move_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "move_down" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = move_window(&Direction::Down, move_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_left_to_left" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Left, -resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_left_to_right" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Left, resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_right_to_right" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Right, resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_right_to_left" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Right, -resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_top_to_top" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Top, resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_top_to_bottom" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Top, -resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_bottom_to_bottom" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Bottom, -resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        } else if op == "resize_bottom_to_top" {
            if let Ok(key) = HotKey::from_str(key) {
                handlers.insert(
                    key.id(),
                    Box::new(move || {
                        let _ = resize(Edge::Bottom, resize_step);
                    }) as Handler,
                );

                hotkey_manager.register(key).unwrap();
            }
        }
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
