use anyhow::Result;
use tray_icon::Icon;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

// Resource ordinals from assets/awake.rc.
const AWAKE: u16 = 1;
const AWAKE_DISPLAY: u16 = 2;

/// Papirus coffee cup at the tray size for the current DPI.
/// Grey when only the system is kept awake, orange when the display is too.
pub fn cup(display: bool) -> Result<Icon> {
  // The manifest makes us DPI aware, so this is already scaled (16 at 100%).
  let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16) as u32;
  let id = if display { AWAKE_DISPLAY } else { AWAKE };
  Ok(Icon::from_resource(id, Some((size, size)))?)
}
