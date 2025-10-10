mod args;

use clap::Parser;
use resize_key_core::{
    frame::{move_window, resize},
    window::ensure_ax_trusted,
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

    match action {
        Action::Move(direction, step) => move_window(&direction.into(), step)?,
        Action::Resize(edge, delta) => resize(edge.into(), delta)?,
    }
    Ok(())
}
