#Requires -Version 7

# Release today's version (YYYY.M.D): set it in Cargo.toml, commit, tag and
# push. The tag triggers .github/workflows/release.yml, which builds and
# publishes the GitHub release.

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location $PSScriptRoot

$branch = git branch --show-current
if ($branch -ne 'master') { throw "Not on master (on '$branch')" }
if (git status --porcelain) { throw 'Uncommitted changes' }

$today = Get-Date
$version = '{0}.{1}.{2}' -f $today.Year, $today.Month, $today.Day
$tag = "v$version"

git fetch --quiet --tags origin
if (git tag --list $tag) { throw "$tag already exists (one release per day)" }

# Only the [package] version starts a line; dependency versions are inline.
$toml = Get-Content Cargo.toml -Raw
$updated = [regex]::new('(?m)^version = ".*"$').Replace($toml, "version = `"$version`"", 1)
if ($updated -eq $toml) { throw 'Version line not found in Cargo.toml' }
Set-Content Cargo.toml $updated -NoNewline
cargo update --workspace --quiet

git commit --quiet --all --message "Release $tag"
git tag --annotate $tag --message $tag
git push --quiet --atomic origin master $tag

Write-Host "Pushed $tag; the release builds at https://github.com/GrzegorzKozub/awake/actions"
