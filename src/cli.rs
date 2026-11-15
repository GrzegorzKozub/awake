use anstyle::{AnsiColor, Effects, Style};
use clap::builder::Styles;
use clap::{Args, Parser, Subcommand};

// Help colors as in cargo, uv and hyperfine, in the normal (not bright) shades.
pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
const STYLES: Styles = Styles::styled()
    .header(HEADER)
    .usage(HEADER)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(LITERAL)
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

/// Keep Windows from going to sleep.
///
/// Sleep settings are never changed: when awake stops ('awake off', tray menu,
/// timer, or the process is killed), Windows goes back to its normal idle
/// behavior and nothing is left running.
///
/// Without a command, shows whether awake is on.
#[derive(Parser, Debug)]
#[command(
    name = "awake",
    bin_name = "awake",
    version,
    about,
    long_about,
    styles = STYLES,
    after_help = examples(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Stay awake in the background and return to the prompt
    On(Opts),

    /// Let the PC sleep again and stop the background instance
    Off,

    /// Show whether awake is on
    Status,

    /// The background instance started by 'awake on' (internal)
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
        args
    }
}

/// The examples section, styled like clap's own headings and literals.
fn examples() -> String {
    const EXAMPLES: [(&str, &str); 6] = [
        ("awake on", "stay awake until stopped"),
        ("awake on -t 2h", "stay awake for two hours"),
        ("awake on -w tidal", "stay awake while TIDAL.exe is running"),
        ("awake on -d", "keep the display on too"),
        ("awake off", "let the PC sleep again"),
        ("awake", "show whether awake is on"),
    ];
    let mut out = format!("{HEADER}Examples:{HEADER:#}");
    for (cmd, what) in EXAMPLES {
        out.push_str(&format!("\n  {LITERAL}{cmd:<22}{LITERAL:#} {what}"));
    }
    out
}
