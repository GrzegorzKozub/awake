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
awake status             on / off
```

`awake on` starts a background instance and returns; running it again replaces
the instance with the new options.

Tray icon: left click toggles keeping the display on, right click opens the menu.

## Build

```
cargo build --release
```

Requires `windres` (MSYS2 / MinGW) to embed the icons and the DPI-aware
manifest. `assets/icons.ps1` regenerates the icons from the
[Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme) panel
icons.
