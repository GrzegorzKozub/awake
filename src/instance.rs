//! Single running instance, found and stopped through named kernel objects.
//!
//! The running instance owns a mutex (released by the OS when it exits) and
//! waits on a manual-reset event that `awake off` signals.

use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use windows_sys::Win32::Foundation::{
  CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Threading::{
  CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, OpenEventW, OpenMutexW, ReleaseMutex,
  SYNCHRONIZATION_SYNCHRONIZE, SetEvent, WaitForSingleObject,
};

const MUTEX: &str = "Local\\awake-running";
const EVENT: &str = "Local\\awake-stop";

fn wide(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}

/// Held by the running instance for its whole lifetime.
pub struct Instance {
  mutex: HANDLE,
  /// Signaled when another `awake` asks this one to stop.
  pub stop: HANDLE,
}

impl Instance {
  pub fn claim() -> Result<Self> {
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 1, wide(MUTEX).as_ptr()) };
    if mutex.is_null() {
      bail!("cannot create instance mutex");
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
      unsafe { CloseHandle(mutex) };
      bail!("awake is already on (use `awake off` first)");
    }
    let stop = unsafe { CreateEventW(std::ptr::null(), 1, 0, wide(EVENT).as_ptr()) };
    if stop.is_null() {
      unsafe {
        ReleaseMutex(mutex);
        CloseHandle(mutex);
      }
      bail!("cannot create stop event");
    }
    Ok(Self { mutex, stop })
  }
}

impl Drop for Instance {
  fn drop(&mut self) {
    unsafe {
      CloseHandle(self.stop);
      ReleaseMutex(self.mutex);
      CloseHandle(self.mutex);
    }
  }
}

pub fn is_running() -> bool {
  let mutex = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, wide(MUTEX).as_ptr()) };
  if mutex.is_null() {
    return false;
  }
  unsafe { CloseHandle(mutex) };
  true
}

/// Ask the running instance to stop and wait until it has exited.
/// Returns false if nothing was running.
pub fn stop(timeout: Duration) -> Result<bool> {
  let mutex = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, wide(MUTEX).as_ptr()) };
  if mutex.is_null() {
    return Ok(false);
  }
  let event = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, wide(EVENT).as_ptr()) };
  if !event.is_null() {
    unsafe {
      SetEvent(event);
      CloseHandle(event);
    }
  }

  // Acquiring the mutex means the old owner released it or died.
  let wait = unsafe { WaitForSingleObject(mutex, timeout.as_millis() as u32) };
  let exited = wait == WAIT_OBJECT_0 || wait == WAIT_ABANDONED;
  unsafe {
    if exited {
      ReleaseMutex(mutex);
    }
    CloseHandle(mutex);
  }
  if !exited {
    bail!("awake did not stop within {}s", timeout.as_secs());
  }
  Ok(true)
}

/// Wait for a freshly spawned instance to claim the mutex.
pub fn wait_started(timeout: Duration, mut alive: impl FnMut() -> bool) -> bool {
  let start = Instant::now();
  while start.elapsed() < timeout {
    if is_running() {
      return true;
    }
    if !alive() {
      return false;
    }
    std::thread::sleep(Duration::from_millis(50));
  }
  false
}
