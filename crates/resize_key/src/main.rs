mod hotkey;
mod preferences;
use std::ffi::c_void;

use objc2::{AnyThread, MainThreadMarker, MainThreadOnly, rc::Retained, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSStatusBar,
};
use objc2_foundation::{NSData, NSSize, ns_string};

use crate::{hotkey::register_hotkeys, preferences::Preferences};

const IMAGE_BYTES: &[u8] = include_bytes!("assets/mono_1.png");
const ICON_SIZE: f64 = 24.0;

fn load_embedded_image() -> Option<Retained<NSImage>> {
    unsafe {
        let data = NSData::dataWithBytes_length(
            IMAGE_BYTES.as_ptr().cast::<c_void>(),
            IMAGE_BYTES.len() as _,
        );
        let image = NSImage::initWithData(NSImage::alloc(), &data)?;
        Some(image)
    }
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let preferences = Preferences::new();

    let Ok(_) = register_hotkeys(&preferences) else {
        eprintln!("Failed to register hotkeys");
        return;
    };

    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    unsafe {
        let status_bar = NSStatusBar::systemStatusBar();
        let item = status_bar.statusItemWithLength(20.);

        if let Some(button) = item.button(mtm) {
            if let Some(image) = load_embedded_image() {
                image.setSize(NSSize::new(ICON_SIZE, ICON_SIZE));
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
