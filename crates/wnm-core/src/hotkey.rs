use anyhow::{anyhow, bail};
use core_graphics::{
    display::{CGPoint, CGSize},
    geometry,
};

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

pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

pub fn resize(edge: Edge, delta: f64) -> anyhow::Result<()> {
    unsafe {
        let Some(win) = get_focused_window() else {
            eprintln!("No focused window.");
            return Ok(());
        };
        if let Some(sz) = get_cgsize(win, get_kAXSizeAttribute()) {
            let new_s = match edge {
                Edge::Right => CGSize::new((sz.width + delta).max(1.0), sz.height),
                Edge::Left => {
                    if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
                        let new_p = CGPoint::new(pos.x + delta, pos.y);
                        let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
                        CGSize::new((sz.width - delta).max(1.0), sz.height)
                    } else {
                        eprintln!("Failed to get window position.");
                        return Ok(());
                    }
                }
                Edge::Top => {
                    if let Some(pos) = get_cgpoint(win, get_kAXPositionAttribute()) {
                        let new_p = CGPoint::new(pos.x, pos.y - delta);
                        let _ = set_cgpoint(win, get_kAXPositionAttribute(), new_p);
                        CGSize::new(sz.width.max(1.0), (sz.height + delta).max(1.0))
                    } else {
                        eprintln!("Failed to get window position.");
                        return Ok(());
                    }
                }
                Edge::Bottom => CGSize::new(sz.width, (sz.height + delta).max(1.0)),
            };
            let _ = set_cgsize(win, get_kAXSizeAttribute(), new_s);
            println!("Resized window to ({}, {}).", new_s.width, new_s.height);
        }
    }
    Ok(())
}
