use anyhow::{Result, bail};
use windows_sys::Win32::System::Power::{
  ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
};

#[derive(Clone, Copy, Debug)]
pub struct Mode {
  pub display: bool,
}

/// Ask Windows to stay awake. Bound to the calling thread and dropped by the
/// OS when the process ends, so a crash or kill can't leave sleep disabled.
pub fn keep_awake(mode: Mode) -> Result<()> {
  let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
  if mode.display {
    flags |= ES_DISPLAY_REQUIRED;
  }
  if unsafe { SetThreadExecutionState(flags) } == 0 {
    bail!("SetThreadExecutionState failed");
  }
  Ok(())
}

pub fn release() {
  unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
}
