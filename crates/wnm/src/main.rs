use std::thread;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use objc2::{sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::ns_string;

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

    let l_l_id = l_l.id();
    let d_d_id = d_d.id();
    let u_u_id = u_u.id();
    let r_r_id = r_r.id();
    let r_l_id = r_l.id();
    let u_d_id = u_d.id();
    let d_u_id = d_u.id();
    let l_r_id = l_r.id();

    thread::spawn(move || {
        let rx = GlobalHotKeyEvent::receiver();
        for ev in rx.iter() {
            if ev.state == global_hotkey::HotKeyState::Pressed {
                if ev.id == l_l_id {
                    println!("left left");
                } else if ev.id == d_d_id {
                    println!("down down");
                } else if ev.id == u_u_id {
                    println!("up up");
                } else if ev.id == r_r_id {
                    println!("right right");
                } else if ev.id == r_l_id {
                    println!("right left");
                } else if ev.id == u_d_id {
                    println!("up down");
                } else if ev.id == d_u_id {
                    println!("down up");
                } else if ev.id == l_r_id {
                    println!("left right");
                }
            }
        }
    });

    let _result = hotkey_manager.register_all(&[l_l, d_d, u_u, r_r, r_l, u_d, d_u, l_r]);

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
