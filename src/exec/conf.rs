use winit::dpi::PhysicalSize;

#[cfg(unix)]
pub const DEFAULT_SHELL: &str = "/usr/bin/sh";
#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";
pub const DEFAULT_ARGS: &[&str] = &[];

pub const FONT_SIZE: i32 = 2;

pub const DISPLAY_SIZE: PhysicalSize<u32> = PhysicalSize::new(1280, 720);
pub const TOUCHPAD_SCROLL_MULTIPLIER: f32 = 0.25;

pub const ICON_SIZE: u32 = 128;
pub const ICON_RGBA: &[u8; (ICON_SIZE * ICON_SIZE * 4) as usize] = include_bytes!("../../docs/assets/shitty.rgba");
