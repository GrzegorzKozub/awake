# awake

Keep Windows from going to sleep, from the command line or the tray.

Uses `SetThreadExecutionState`, so power settings are never changed: when awake
stops (or is killed), Windows goes back to its normal idle behavior.

## Usage

```
awake on                 stay awake until `awake off`
awake on -t 2h           stay awake for two hours
awake on -w tidal        stay awake while TIDAL.exe is running
awake on -d              keep the display on too
awake off                let the PC sleep again
awake status             Awake on / Awake off (also plain `awake`)
```

`awake on` starts a background instance and returns; running it again replaces
the instance with the new options.

Tray icon: left click toggles keeping the display on, right click opens the menu.

## Install

Download `awake.exe` from the
[latest release](https://github.com/GrzegorzKozub/awake/releases/latest) and
put it on your `PATH`.

## Build

```
cargo build --release
```

Embedding the icons and the DPI-aware manifest needs a resource compiler:
`windres` (MSYS2 / MinGW) for the GNU toolchain, or `rc.exe` (Windows SDK) for
MSVC. `assets/icons.ps1` regenerates the icons from the
[Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme) panel
icons.

## Release

Versions are dates: `YYYY.M.D`, at most one release per day.

```
./release.ps1
```

sets today's version in `Cargo.toml`, commits, and pushes a `v` tag. The tag
runs the release workflow, which checks formatting and lints, builds a
statically linked MSVC exe, and publishes
`awake-vYYYY.M.D-x86_64-pc-windows-msvc.zip`.

## License

[GPL-3.0](LICENSE), as are the bundled Papirus icons.
