mod args;
mod extern_c;

use anyhow::Ok;
use clap::Parser;
use core_graphics::geometry::{CGPoint, CGSize};

use crate::{
    args::{Action, Args, parse_move, parse_resize},
    extern_c::{
        ensure_ax_trusted, get_cgpoint, get_cgsize, get_focused_window, get_kAXPositionAttribute,
        get_kAXSizeAttribute, set_cgpoint, set_cgsize,
    },
};

fn main() -> anyhow::Result<()> {
    if !ensure_ax_trusted() {
        eprintln!("Enable Accessibility permission for this app, then run again.");
        return Ok(());
    }

    let args = Args::parse();

    let action = match (args.r#move, args.resize) {
        (Some(m), None) => parse_move(&m).map_err(anyhow::Error::msg)?,
        (None, Some(r)) => parse_resize(&r).map_err(anyhow::Error::msg)?,
        (None, None) => return Err(anyhow::Error::msg("specify either --move or --resize")),
        (Some(_), Some(_)) => return Err(anyhow::Error::msg("use only one of --move or --resize")),
    };

    unsafe {
        let Some(win) = get_focused_window() else {
            eprintln!("No focused window.");
            return Ok(());
        };

        match action {
            Action::Move(direction, step) => {
                if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
                    let new_p = match direction {
                        args::Direction::Right => CGPoint::new(pos.x + step, pos.y),
                        args::Direction::Left => CGPoint::new(pos.x - step, pos.y),
                        args::Direction::Up => CGPoint::new(pos.x, pos.y - step),
                        args::Direction::Down => CGPoint::new(pos.x, pos.y + step),
                    };
                    let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
                    println!("Moved window to ({}, {}).", new_p.x, new_p.y);
                } else {
                    eprintln!("Failed to get window position.");
                }
            }
            Action::Resize(edge, delta) => {
                if let Some(sz) = get_cgsize(win, get_kAXSizeAttribute()) {
                    let new_s = match edge {
                        args::Edge::Right => CGSize::new((sz.width + delta).max(1.0), sz.height),
                        args::Edge::Left => {
                            if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
                                let new_p = CGPoint::new(pos.x + delta, pos.y);
                                let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
                                CGSize::new((sz.width - delta).max(1.0), sz.height)
                            } else {
                                eprintln!("Failed to get window position.");
                                return Ok(());
                            }
                        }
                        args::Edge::Top => {
                            if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
                                let new_p = CGPoint::new(pos.x, pos.y + delta);
                                let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
                                CGSize::new(sz.width.max(1.0), (sz.height + delta).max(1.0))
                            } else {
                                eprintln!("Failed to get window position.");
                                return Ok(());
                            }
                        }
                        args::Edge::Bottom => CGSize::new(sz.width, (sz.height + delta).max(1.0)),
                    };
                    let _ = set_cgsize(win, get_kAXSizeAttribute(), new_s);
                    println!("Resized window to ({}, {}).", new_s.width, new_s.height);
                } else {
                    eprintln!("Failed to get window size.");
                }
            }
        }
        // println!("Moved right by {step}pt and widened by {step}pt.");
    }
    Ok(())
}
