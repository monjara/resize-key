mod preferences;
use std::{collections::HashMap, str::FromStr, sync::Arc, thread};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use objc2::{AnyThread, MainThreadMarker, MainThreadOnly, rc::Retained, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::{NSData, NSSize, ns_string};
use resize_key_core::frame::{Direction, Edge, move_window_nswindow_style, resize};

use crate::preferences::{Operation, Preferences};

const IMAGE_BYTES: &[u8] = include_bytes!("assets/mono_1.png");

fn load_embedded_image() -> Option<Retained<NSImage>> {
    unsafe {
        let data =
            NSData::dataWithBytes_length(IMAGE_BYTES.as_ptr() as *const _, IMAGE_BYTES.len() as _);
        let image = NSImage::initWithData(NSImage::alloc(), &data)?;
        Some(image)
    }
}

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
        let item = status_bar.statusItemWithLength(20.);

        if let Some(button) = item.button(mtm) {
            if let Some(image) = load_embedded_image() {
                image.setSize(NSSize::new(24., 24.));
                button.setImage(Some(&image));
            } else {
                button.setTitle(ns_string!("R"));
            }
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
