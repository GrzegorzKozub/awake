use clap::{Args, Parser, Subcommand};

/// Keep Windows from going to sleep.
///
/// Sleep settings are never changed: when awake stops (`awake off`, tray menu,
/// timer, or the process is killed), Windows goes back to its normal idle
/// behavior and nothing is left running.
#[derive(Parser, Debug)]
#[command(
  version,
  about,
  long_about,
  after_help = "\x1b[1;4mExamples:\x1b[0m
  awake on                 stay awake until `awake off`
  awake on -t 2h           stay awake for two hours
  awake on -w tidal        stay awake while TIDAL.exe is running
  awake on -d              keep the display on too
  awake off                let the PC sleep again"
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// Stay awake in the background and return to the prompt
  #[command(visible_aliases = ["stay", "start"])]
  On(Opts),

  /// Let the PC sleep again and stop the background instance
  #[command(visible_aliases = ["sleep", "stop"])]
  Off,

  /// Show whether awake is on
  Status,

  /// The background instance started by `awake on` (internal)
  #[command(hide = true)]
  Serve(Opts),
}

#[derive(Args, Debug, Clone)]
pub struct Opts {
  /// Keep the display on as well
  #[arg(short, long)]
  pub display: bool,

  /// Stop after a duration (e.g. 90m, 1h30m)
  #[arg(short = 't', long = "for", value_name = "DURATION")]
  pub r#for: Option<humantime::Duration>,

  /// Stop when this process exits (e.g. tidal, TIDAL.exe)
  #[arg(short, long = "while", value_name = "PROCESS")]
  pub r#while: Option<String>,

  /// Don't show a tray icon
  #[arg(long)]
  pub no_tray: bool,
}

impl Opts {
  /// Arguments that reproduce these options for `awake serve`.
  pub fn to_args(&self) -> Vec<String> {
    let mut args = Vec::new();
    if self.display {
      args.push("--display".into());
    }
    if let Some(d) = &self.r#for {
      args.extend(["--for".into(), d.to_string()]);
    }
    if let Some(p) = &self.r#while {
      args.extend(["--while".into(), p.clone()]);
    }
    if self.no_tray {
      args.push("--no-tray".into());
    }
    args
  }
}
