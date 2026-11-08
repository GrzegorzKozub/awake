# regenerates awake.ico and awake-display.ico from papirus panel icons
# https://github.com/PapirusDevelopmentTeam/papirus-icon-theme

$lib = 'mingw-w64-ucrt-x86_64-librsvg'
pacman -Qq $lib 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { pacman -S --noconfirm $lib }

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$url = 'https://raw.githubusercontent.com/PapirusDevelopmentTeam/papirus-icon-theme/refs/heads/master/Papirus'
$icon = 'caffeine-cup-full'
# png size -> papirus source size (papirus ships 16, 22 and 24 panel icons)
$sizes = (16, 16), (20, 22), (24, 24), (32, 24), (40, 24), (48, 24), (64, 24)

Push-Location $PSScriptRoot
try {
  foreach ($src in $sizes.ForEach({ $_[1] }) | Sort-Object -Unique) {
    curl.exe -fsL --retry 32 --retry-all-errors --retry-delay 2 "$url/$($src)x$($src)/panel/$icon.svg" -o "$src.svg"
  }

  # papirus text color; display mode uses papirus orange (NeutralText)
  foreach ($variant in @{ name = 'awake'; color = '#dfdfdf' }, @{ name = 'awake-display'; color = '#ff9800' }) {
    $pngs = foreach ($pair in $sizes) {
      $size, $src = $pair
      $svg = "$src.svg"
      (Get-Content $svg -Raw).Replace('#dfdfdf', $variant.color) | Set-Content -NoNewline 'tmp.svg'
      rsvg-convert --width $size --height $size 'tmp.svg' -o "$size.png"
      "$size.png"
    }
    magick @pngs "$($variant.name).ico"
    Remove-Item $pngs
  }
} finally {
  Remove-Item *.svg -ErrorAction SilentlyContinue
  Pop-Location
}
