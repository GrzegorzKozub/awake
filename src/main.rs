mod cli;
mod icon;
mod instance;
mod power;
mod process;

use std::time::{Duration, Instant};

use anstream::{eprintln, println};
use anyhow::{Context, Result, bail};
use clap::Parser;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::UI::WindowsAndMessaging::{
  DispatchMessageW, MSG, MsgWaitForMultipleObjects, PM_REMOVE, PeekMessageW, QS_ALLINPUT,
  TranslateMessage, WM_QUIT,
};

use cli::{Cli, Command, Opts};
use instance::Instance;
use power::Mode;

const TICK: Duration = Duration::from_secs(1);
const PROCESS_POLL: Duration = Duration::from_secs(5);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const START_TIMEOUT: Duration = Duration::from_secs(5);

const OK: anstyle::Style = anstyle::AnsiColor::Green.on_default().bold();
const OFF: anstyle::Style = anstyle::AnsiColor::Yellow.on_default().bold();
const ERR: anstyle::Style = anstyle::AnsiColor::Red.on_default().bold();

fn main() {
  let result = match Cli::parse().command {
    Command::On(opts) => on(&opts),
    Command::Off => off(),
    Command::Status => status(),
    Command::Serve(opts) => serve(&opts),
  };
  if let Err(err) = result {
    eprintln!("{ERR}error:{ERR:#} {err:#}");
    std::process::exit(1);
  }
}

/// Start a detached `awake serve` (replacing any running one) and return.
fn on(opts: &Opts) -> Result<()> {
  use std::os::windows::process::CommandExt;
  const DETACHED_PROCESS: u32 = 0x0000_0008;
  const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

  check_process(opts)?;
  let replaced = instance::stop(STOP_TIMEOUT)?;

  let exe = std::env::current_exe().context("cannot locate own executable")?;
  let mut child = std::process::Command::new(exe)
    .arg("serve")
    .args(opts.to_args())
    .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
    .spawn()
    .context("cannot start background process")?;

  let started = instance::wait_started(START_TIMEOUT, || matches!(child.try_wait(), Ok(None)));
  if !started {
    match child.try_wait() {
      Ok(Some(code)) => bail!("background process exited early ({code})"),
      _ => bail!("background process did not start"),
    }
  }

  let verb = if replaced { "restarted" } else { "on" };
  println!("{OK}Awake {verb}{OK:#} {}", describe(opts));
  println!("`awake off` to let the PC sleep again");
  Ok(())
}

fn off() -> Result<()> {
  if instance::stop(STOP_TIMEOUT)? {
    println!("{OFF}Sleep allowed{OFF:#} (awake stopped)");
  } else {
    println!("awake was not on");
  }
  Ok(())
}

fn status() -> Result<()> {
  if instance::is_running() {
    println!("{OK}on{OK:#}");
  } else {
    println!("{OFF}off{OFF:#}");
  }
  Ok(())
}

fn check_process(opts: &Opts) -> Result<()> {
  if let Some(name) = &opts.r#while
    && !process::is_running(name)?
  {
    bail!("no process named '{name}' is running");
  }
  Ok(())
}

/// The background instance: stay awake on this thread until stopped.
fn serve(opts: &Opts) -> Result<()> {
  check_process(opts)?;
  let instance = Instance::claim()?;

  let mut mode = Mode::from(opts);
  power::keep_awake(mode)?;

  let deadline = opts.r#for.map(|d| Instant::now() + *d);
  let tray = if opts.no_tray {
    None
  } else {
    Some(Tray::new(mode)?)
  };

  let mut msg: MSG = unsafe { std::mem::zeroed() };
  let mut last_poll = Instant::now();
  loop {
    let wait = unsafe {
      MsgWaitForMultipleObjects(1, &instance.stop, 0, TICK.as_millis() as u32, QS_ALLINPUT)
    };
    if wait == WAIT_OBJECT_0 {
      break; // awake off
    }

    let mut stop = false;
    while unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
      stop |= msg.message == WM_QUIT;
      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
    }

    if let Some(tray) = &tray {
      while let Ok(event) = MenuEvent::receiver().try_recv() {
        if event.id == tray.quit.id() {
          stop = true;
        } else if event.id == tray.display.id() {
          mode.display = tray.display.is_checked();
          power::keep_awake(mode)?;
          tray.update(mode);
        }
      }
      // Left click toggles the display; the menu is on right click only.
      while let Ok(event) = TrayIconEvent::receiver().try_recv() {
        if let TrayIconEvent::Click {
          button: MouseButton::Left,
          button_state: MouseButtonState::Up,
          ..
        } = event
        {
          mode.display = !mode.display;
          tray.display.set_checked(mode.display);
          power::keep_awake(mode)?;
          tray.update(mode);
        }
      }
    }

    if stop || deadline.is_some_and(|d| Instant::now() >= d) {
      break;
    }
    if let Some(name) = &opts.r#while
      && last_poll.elapsed() >= PROCESS_POLL
    {
      last_poll = Instant::now();
      if !process::is_running(name)? {
        break;
      }
    }
    if let Some(tray) = &tray {
      tray.refresh(&status_line(opts, deadline));
    }
  }

  power::release();
  drop(tray);
  drop(instance);
  Ok(())
}

impl From<&Opts> for Mode {
  fn from(opts: &Opts) -> Self {
    Mode {
      display: opts.display,
    }
  }
}

struct Tray {
  icon: TrayIcon,
  status: MenuItem,
  shown: std::cell::RefCell<String>,
  display: CheckMenuItem,
  quit: MenuItem,
}

impl Tray {
  fn new(mode: Mode) -> Result<Self> {
    let display = CheckMenuItem::new("Keep &display on", true, mode.display, None);
    let quit = MenuItem::new("Let system &sleep", true, None);
    let status = MenuItem::new("Awake", false, None);
    let menu = Menu::new();
    menu.append_items(&[&status, &display, &quit])?;
    let icon = TrayIconBuilder::new()
      .with_menu(Box::new(menu))
      .with_menu_on_left_click(false)
      .with_icon(icon::cup(mode.display)?)
      .with_tooltip("Awake")
      .build()
      .context("cannot create tray icon")?;
    Ok(Self {
      icon,
      status,
      shown: Default::default(),
      display,
      quit,
    })
  }

  fn update(&self, mode: Mode) {
    if let Ok(icon) = icon::cup(mode.display) {
      let _ = self.icon.set_icon(Some(icon));
    }
  }

  /// Show `text` in the status menu item and tooltip, if it changed.
  fn refresh(&self, text: &str) {
    if *self.shown.borrow() == text {
      return;
    }
    self.status.set_text(text);
    let _ = self.icon.set_tooltip(Some(text));
    *self.shown.borrow_mut() = text.to_owned();
  }
}

/// e.g. "system + display, for 1h 30m, while tidal runs"
fn describe(opts: &Opts) -> String {
  let mut parts = vec![
    if opts.display {
      "system + display"
    } else {
      "system"
    }
    .to_owned(),
  ];
  if let Some(d) = &opts.r#for {
    parts.push(format!("for {d}"));
  }
  if let Some(p) = &opts.r#while {
    parts.push(format!("while {p} runs"));
  }
  parts.join(", ")
}

/// Tray status line, e.g. "Awake for 1h 20m, while tidal runs".
fn status_line(opts: &Opts, deadline: Option<Instant>) -> String {
  let mut parts = Vec::new();
  if let Some(d) = deadline {
    parts.push(format!("for {}", remaining(d)));
  }
  if let Some(p) = &opts.r#while {
    parts.push(format!("while {p} runs"));
  }
  if parts.is_empty() {
    parts.push("until stopped".to_owned());
  }
  format!("Awake {}", parts.join(", "))
}

fn remaining(deadline: Instant) -> String {
  let left = deadline.saturating_duration_since(Instant::now());
  let secs = left.as_millis().div_ceil(1000) as u64;
  // Round up to whole minutes once past a minute, so the text stays short.
  let secs = if secs >= 60 {
    secs.div_ceil(60) * 60
  } else {
    secs
  };
  humantime::format_duration(Duration::from_secs(secs)).to_string()
}
