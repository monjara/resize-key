mod preferences;
use std::{collections::HashMap, sync::Arc, thread};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
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
    println!("Preferences: {:#?}", preferences);

    // todo
    let resize_step = preferences.resize_step;
    let resize_l_l = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyH);
    let resize_d_d = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyJ);
    let resize_u_u = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyK);
    let resize_r_r = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyL);
    let resize_r_l = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyH);
    let resize_u_d = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyJ);
    let resize_d_u = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyK);
    let resize_l_r = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyL);
    let move_step = preferences.move_step;
    let move_l = HotKey::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::KeyH);
    let move_r = HotKey::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::KeyL);
    let move_u = HotKey::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::KeyK);
    let move_d = HotKey::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::KeyJ);

    type Handler = Box<dyn Fn() + Send + Sync + 'static>;

    let handlers: Arc<HashMap<u32, Handler>> = Arc::new(HashMap::from([
        (
            move_l.id(),
            Box::new(move || unsafe {
                let _ = move_window(&Direction::Left, move_step);
            }) as Handler,
        ),
        (
            move_r.id(),
            Box::new(move || unsafe {
                let _ = move_window(&Direction::Right, move_step);
            }) as Handler,
        ),
        (
            move_u.id(),
            Box::new(move || unsafe {
                let _ = move_window(&Direction::Up, move_step);
            }) as Handler,
        ),
        (
            move_d.id(),
            Box::new(move || unsafe {
                let _ = move_window(&Direction::Down, move_step);
            }) as Handler,
        ),
        (
            resize_l_l.id(),
            Box::new(move || {
                let _ = resize(Edge::Left, -resize_step);
            }) as Handler,
        ),
        (
            resize_d_d.id(),
            Box::new(move || {
                let _ = resize(Edge::Bottom, resize_step);
            }),
        ),
        (
            resize_u_u.id(),
            Box::new(move || {
                let _ = resize(Edge::Top, resize_step);
            }),
        ),
        (
            resize_r_r.id(),
            Box::new(move || {
                let _ = resize(Edge::Right, resize_step);
            }),
        ),
        (
            resize_r_l.id(),
            Box::new(move || {
                let _ = resize(Edge::Right, -resize_step);
            }),
        ),
        (
            resize_u_d.id(),
            Box::new(move || {
                let _ = resize(Edge::Top, -resize_step);
            }),
        ),
        (
            resize_d_u.id(),
            Box::new(move || {
                let _ = resize(Edge::Bottom, -resize_step);
            }),
        ),
        (
            resize_l_r.id(),
            Box::new(move || {
                let _ = resize(Edge::Left, resize_step);
            }),
        ),
    ]));

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

    if let Err(e) = hotkey_manager.register_all(&[
        resize_l_l, resize_d_d, resize_u_u, resize_r_r, resize_r_l, resize_u_d, resize_d_u,
        resize_l_r, move_l, move_r, move_u, move_d,
    ]) {
        eprintln!("Failed to register hotkeys: {}", e);
        std::process::exit(1);
    }

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
