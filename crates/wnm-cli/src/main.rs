mod args;

use clap::Parser;
use core_graphics::geometry::CGPoint;
use wnm_core::{
    frame::resize,
    window::{
        ensure_ax_trusted, get_cgpoint, get_focused_window, get_kAXPositionAttribute, set_cgpoint,
    },
};

use crate::args::{Action, Args, parse_move, parse_resize};

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
                resize(edge.into(), delta)?;
            }
        }
        // println!("Moved right by {step}pt and widened by {step}pt.");
    }
    Ok(())
}
