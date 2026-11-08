use anyhow::{Result, bail};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
  CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

/// Is a process with this image name running? Case-insensitive, `.exe` optional.
pub fn is_running(name: &str) -> Result<bool> {
  let wanted = normalize(name);
  let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
  if snapshot == INVALID_HANDLE_VALUE {
    bail!("cannot list processes");
  }

  let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
  entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
  let mut found = false;
  let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
  while ok {
    let len = entry
      .szExeFile
      .iter()
      .position(|&c| c == 0)
      .unwrap_or(entry.szExeFile.len());
    if normalize(&String::from_utf16_lossy(&entry.szExeFile[..len])) == wanted {
      found = true;
      break;
    }
    ok = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
  }

  unsafe { CloseHandle(snapshot) };
  Ok(found)
}

fn normalize(name: &str) -> String {
  let lower = name.to_lowercase();
  lower
    .strip_suffix(".exe")
    .map(str::to_owned)
    .unwrap_or(lower)
}
