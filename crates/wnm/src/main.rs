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
use wnm_core::hotkey::{Edge, resize};

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();

    let l_l = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyH);
    let d_d = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyJ);
    let u_u = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyK);
    let r_r = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyL);
    let r_l = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyH);
    let u_d = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyJ);
    let d_u = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyK);
    let l_r = HotKey::new(Some(Modifiers::META | Modifiers::CONTROL), Code::KeyL);

    type Handler = Box<dyn Fn() + Send + Sync + 'static>;

    let handlers: Arc<HashMap<u32, Handler>> = Arc::new(HashMap::from([
        (
            l_l.id(),
            Box::new(|| {
                let _ = resize(Edge::Left, -100.);
            }) as Handler,
        ),
        (
            d_d.id(),
            Box::new(|| {
                let _ = resize(Edge::Bottom, 100.);
            }),
        ),
        (
            u_u.id(),
            Box::new(|| {
                let _ = resize(Edge::Top, 100.);
            }),
        ),
        (
            r_r.id(),
            Box::new(|| {
                let _ = resize(Edge::Right, 100.);
            }),
        ),
        (
            r_l.id(),
            Box::new(|| {
                let _ = resize(Edge::Right, -100.);
            }),
        ),
        (
            u_d.id(),
            Box::new(|| {
                let _ = resize(Edge::Top, -100.);
            }),
        ),
        (
            d_u.id(),
            Box::new(|| {
                let _ = resize(Edge::Bottom, -100.);
            }),
        ),
        (
            l_r.id(),
            Box::new(|| {
                let _ = resize(Edge::Left, 100.);
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

    if let Err(e) = hotkey_manager.register_all(&[l_l, d_d, u_u, r_r, r_l, u_d, d_u, l_r]) {
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
