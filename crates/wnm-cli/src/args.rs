use std::str::FromStr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Move the whole window: --move <dir> <step>
    /// <dir> = right|left|up|down, <step> = integer (points)
    #[arg(long = "move", value_names = ["DIR", "STEP"], num_args = 2)]
    pub(crate) r#move: Option<Vec<String>>,

    /// Resize by moving an edge: --resize <edge> <delta>
    /// <edge> = left|right|top|bottom, <delta> = integer (points; sign = direction)
    #[arg(long = "resize", value_names = ["EDGE", "DELTA"], num_args = 2, allow_hyphen_values = true)]
    pub(crate) resize: Option<Vec<String>>,
}

pub(crate) enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub(crate) enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl From<Edge> for wnm_core::hotkey::Edge {
    fn from(value: Edge) -> Self {
        match value {
            Edge::Left => wnm_core::hotkey::Edge::Left,
            Edge::Right => wnm_core::hotkey::Edge::Right,
            Edge::Top => wnm_core::hotkey::Edge::Top,
            Edge::Bottom => wnm_core::hotkey::Edge::Bottom,
        }
    }
}

pub(crate) enum Action {
    Move(Direction, f64),
    Resize(Edge, f64),
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "r" | "right" => Ok(Direction::Right),
            "l" | "left" => Ok(Direction::Left),
            "u" | "up" => Ok(Direction::Up),
            "d" | "down" => Ok(Direction::Down),
            _ => Err(format!("invalid dir: {s}")),
        }
    }
}

impl FromStr for Edge {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "l" | "left" => Ok(Edge::Left),
            "r" | "right" => Ok(Edge::Right),
            "t" | "top" => Ok(Edge::Top),
            "b" | "bottom" => Ok(Edge::Bottom),
            _ => Err(format!("invalid edge: {s}")),
        }
    }
}

pub(crate) fn parse_move(vals: &[String]) -> Result<Action, String> {
    if vals.len() != 2 {
        return Err("needs: --move <dir> <step>".into());
    }
    let dir = Direction::from_str(&vals[0])?;
    let step: f64 = vals[1]
        .parse()
        .map_err(|_| "STEP must be a floating point number")?;
    Ok(Action::Move(dir, step))
}

pub(crate) fn parse_resize(vals: &[String]) -> Result<Action, String> {
    if vals.len() != 2 {
        return Err("needs: --resize <edge> <delta>".into());
    }
    let edge = Edge::from_str(&vals[0])?;
    let delta: f64 = vals[1]
        .parse()
        .map_err(|_| "DELTA must be a floating point number")?;
    Ok(Action::Resize(edge, delta))
}
