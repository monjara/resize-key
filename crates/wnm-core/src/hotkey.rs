use anyhow::{anyhow, bail};
use core_graphics::geometry;

use crate::window::{
    get_cgpoint, get_cgsize, get_focused_window, get_kAXPositionAttribute, get_kAXSizeAttribute,
    set_cgpoint, set_cgsize,
};

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub fn get_frame() -> anyhow::Result<Frame> {
    unsafe {
        let win = get_focused_window().ok_or_else(|| anyhow!("no focused window"))?;
        let p = get_cgpoint(win, get_kAXPositionAttribute())
            .ok_or_else(|| anyhow!("get position failed"))?;
        let s =
            get_cgsize(win, get_kAXSizeAttribute()).ok_or_else(|| anyhow!("get size failed"))?;
        Ok(Frame {
            x: p.x,
            y: p.y,
            w: s.width,
            h: s.height,
        })
    }
}

pub fn set_frame(f: Frame) -> anyhow::Result<()> {
    unsafe {
        let win = get_focused_window().ok_or_else(|| anyhow!("no focused window"))?;
        // 左/上辺固定っぽく見せたいならサイズ→位置の順が揺れにくい
        if !set_cgsize(win, get_kAXSizeAttribute(), geometry::CGSize::new(f.w, f.h)) {
            bail!("set size failed")
        }
        if !set_cgpoint(
            win,
            get_kAXPositionAttribute(),
            geometry::CGPoint::new(f.x, f.y),
        ) {
            bail!("set position failed")
        }
        Ok(())
    }
}
// 便利API（CLI/GUIから呼ぶ）
pub fn move_by(dx: f64, dy: f64) -> anyhow::Result<()> {
    let mut f = get_frame()?;
    f.x += dx;
    f.y += dy;
    set_frame(f)
}
pub fn resize_from_left(delta: f64) -> anyhow::Result<()> {
    let mut f = get_frame()?;
    f.x -= delta;
    f.w += delta;
    set_frame(f)
}
pub fn resize_from_right(delta: f64) -> anyhow::Result<()> {
    let mut f = get_frame()?;
    f.w += delta;
    set_frame(f)
}
pub fn resize_from_top(delta: f64) -> anyhow::Result<()> {
    let mut f = get_frame()?;
    f.h += delta;
    set_frame(f)
}
pub fn resize_from_bottom(delta: f64) -> anyhow::Result<()> {
    let mut f = get_frame()?;
    f.y -= delta;
    f.h += delta;
    set_frame(f)
}
