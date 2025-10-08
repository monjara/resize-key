use anyhow::{anyhow, bail};
use core_graphics::display::{CGPoint, CGSize};

use crate::window::{
    __AXUIElement, get_cgpoint, get_cgsize, get_focused_window, get_kAXPositionAttribute,
    get_kAXSizeAttribute, set_cgpoint, set_cgsize,
};

type AXUIElementRef = *const __AXUIElement;

/// ウィンドウの最小サイズ（ピクセル）
const MIN_WINDOW_SIZE: f64 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Frame {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_position_and_size(position: CGPoint, size: CGSize) -> Self {
        Self {
            x: position.x,
            y: position.y,
            w: size.width,
            h: size.height,
        }
    }

    pub fn position(&self) -> CGPoint {
        CGPoint::new(self.x, self.y)
    }

    pub fn size(&self) -> CGSize {
        CGSize::new(self.w, self.h)
    }
}

pub fn get_frame() -> anyhow::Result<Frame> {
    unsafe {
        let win = get_focused_window().ok_or_else(|| anyhow!("No focused window"))?;

        let position = get_cgpoint(win, get_kAXPositionAttribute())
            .ok_or_else(|| anyhow!("Failed to get window position"))?;

        let size = get_cgsize(win, get_kAXSizeAttribute())
            .ok_or_else(|| anyhow!("Failed to get window size"))?;

        Ok(Frame::from_position_and_size(position, size))
    }
}

pub fn set_frame(frame: Frame) -> anyhow::Result<()> {
    unsafe {
        let win = get_focused_window().ok_or_else(|| anyhow!("No focused window"))?;

        // サイズ→位置の順で設定することで、左/上辺固定っぽく見せる
        if !set_cgsize(win, get_kAXSizeAttribute(), frame.size()) {
            bail!("Failed to set window size");
        }

        if !set_cgpoint(win, get_kAXPositionAttribute(), frame.position()) {
            bail!("Failed to set window position");
        }

        Ok(())
    }
}

/// ウィンドウのリサイズ対象となる辺
#[derive(Debug, Clone, Copy)]
pub enum Edge {
    /// 左辺
    Left,
    /// 右辺
    Right,
    /// 上辺
    Top,
    /// 下辺
    Bottom,
}

pub fn resize(edge: Edge, delta: f64) -> anyhow::Result<()> {
    unsafe {
        let win = get_focused_window().ok_or_else(|| anyhow!("No focused window"))?;

        let current_size = get_cgsize(win, get_kAXSizeAttribute())
            .ok_or_else(|| anyhow!("Failed to get window size"))?;

        let new_size = match edge {
            Edge::Right => resize_right(current_size, delta),
            Edge::Left => resize_left(win, current_size, delta)?,
            Edge::Top => resize_top(win, current_size, delta)?,
            Edge::Bottom => resize_bottom(current_size, delta),
        };

        if !set_cgsize(win, get_kAXSizeAttribute(), new_size) {
            bail!("Failed to set window size");
        }

        println!(
            "Resized window to ({}, {})",
            new_size.width, new_size.height
        );
        Ok(())
    }
}

fn resize_right(current_size: CGSize, delta: f64) -> CGSize {
    CGSize::new(
        (current_size.width + delta).max(MIN_WINDOW_SIZE),
        current_size.height,
    )
}

fn resize_left(win: AXUIElementRef, current_size: CGSize, delta: f64) -> anyhow::Result<CGSize> {
    unsafe {
        let current_pos = get_cgpoint(win, get_kAXPositionAttribute())
            .ok_or_else(|| anyhow!("Failed to get window position"))?;

        let new_pos = CGPoint::new(current_pos.x + delta, current_pos.y);
        if !set_cgpoint(win, get_kAXPositionAttribute(), new_pos) {
            bail!("Failed to set window position");
        }

        Ok(CGSize::new(
            (current_size.width - delta).max(MIN_WINDOW_SIZE),
            current_size.height,
        ))
    }
}

fn resize_top(win: AXUIElementRef, current_size: CGSize, delta: f64) -> anyhow::Result<CGSize> {
    unsafe {
        let current_pos = get_cgpoint(win, get_kAXPositionAttribute())
            .ok_or_else(|| anyhow!("Failed to get window position"))?;

        let new_pos = CGPoint::new(current_pos.x, current_pos.y - delta);
        if !set_cgpoint(win, get_kAXPositionAttribute(), new_pos) {
            bail!("Failed to set window position");
        }

        Ok(CGSize::new(
            current_size.width.max(MIN_WINDOW_SIZE),
            (current_size.height + delta).max(MIN_WINDOW_SIZE),
        ))
    }
}

fn resize_bottom(current_size: CGSize, delta: f64) -> CGSize {
    CGSize::new(
        current_size.width,
        (current_size.height + delta).max(MIN_WINDOW_SIZE),
    )
}

pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub fn move_window(direction: &Direction, step: f64) -> anyhow::Result<()> {
    unsafe {
        let window = get_focused_window().ok_or_else(|| anyhow!("No focused window"))?;
        if let Some(pos) = get_cgpoint(window, get_kAXPositionAttribute()) {
            let new_p = match direction {
                Direction::Right => CGPoint::new(pos.x + step, pos.y),
                Direction::Left => CGPoint::new(pos.x - step, pos.y),
                Direction::Up => CGPoint::new(pos.x, pos.y - step),
                Direction::Down => CGPoint::new(pos.x, pos.y + step),
            };
            let _ = set_cgpoint(window, get_kAXPositionAttribute(), new_p);
            println!("Moved window to ({}, {}).", new_p.x, new_p.y);
        }
    }

    Ok(())
}
