param([string]$Toolchain = '', [switch]$Check, [switch]$Zip)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
$previousRustFlags = $env:RUSTFLAGS
$previousPath = $env:PATH
$previousCargoHome = $env:CARGO_HOME
$previousRustupHome = $env:RUSTUP_HOME
try {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    $cargoExe = if ($cargoCommand) { $cargoCommand.Source } else { Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe' }
    $localCargo = Join-Path $projectRoot 'target/tools/cargo/bin/cargo.exe'
    if (-not (Test-Path -LiteralPath $cargoExe) -and (Test-Path -LiteralPath $localCargo)) {
        $cargoExe = $localCargo
        $env:CARGO_HOME = Join-Path $projectRoot 'target/tools/cargo'
        $env:RUSTUP_HOME = Join-Path $projectRoot 'target/tools/rustup'
        $env:PATH = "$(Split-Path -Parent $cargoExe);$(Join-Path $projectRoot 'target/tools/mingw64/bin');$previousPath"
        if (-not $Toolchain) { $Toolchain = 'stable-x86_64-pc-windows-gnu' }
    }
    if (-not (Test-Path -LiteralPath $cargoExe)) { throw 'Install Rust from https://rustup.rs, then run this script again.' }
    $cargoArgs = @()
    if ($Toolchain) { $cargoArgs = @("+$Toolchain") }
    & $cargoExe @cargoArgs fetch --locked
    if ($LASTEXITCODE -ne 0) { throw 'Dependency download failed.' }

    # Some minimal Rust GNU installations omit the ordinary imm32 import library.
    # winapi already supplies the same Windows system DLL imports under another name.
    if ($Toolchain -like '*windows-gnu') {
        $cargoCache = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
        $targetArch = if ($Toolchain -like '*i686*') { 'i686' } else { 'x86_64' }
        $importLibrary = Get-ChildItem -LiteralPath (Join-Path $cargoCache 'registry/src') -Filter libwinapi_imm32.a -Recurse |
            Where-Object { $_.FullName -like "*winapi-$targetArch-pc-windows-gnu-*" } |
            Select-Object -First 1
        if ($importLibrary) {
            $nativeDir = Join-Path $projectRoot 'target/native'
            New-Item -ItemType Directory -Force -Path $nativeDir | Out-Null
            Copy-Item -LiteralPath $importLibrary.FullName -Destination (Join-Path $nativeDir 'libimm32.a')
            $env:RUSTFLAGS = "$previousRustFlags -Lnative=target/native".Trim()
        }
    }
    if ($Check) {
        & $cargoExe @cargoArgs fmt --all -- --check
        if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed.' }
        & $cargoExe @cargoArgs test --locked --all-targets
        if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }
        & $cargoExe @cargoArgs clippy --locked --all-targets -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw 'Clippy failed.' }
    }
    & $cargoExe @cargoArgs build --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed. MSVC needs the Visual C++ build tools; GNU needs MinGW-w64.' }
    $destination = Join-Path $projectRoot 'dist/MGame-Overdrive'
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    Copy-Item -LiteralPath 'target/release/mgame.exe' -Destination $destination
    Copy-Item -LiteralPath 'README.md' -Destination $destination
    Copy-Item -LiteralPath 'MapEditor.cmd' -Destination $destination
    New-Item -ItemType Directory -Force -Path (Join-Path $destination 'Maps') | Out-Null
    Get-ChildItem -LiteralPath 'Maps' -File | Copy-Item -Destination (Join-Path $destination 'Maps')
    New-Item -ItemType Directory -Force -Path (Join-Path $destination 'levels') | Out-Null
    Get-ChildItem -LiteralPath 'levels' -File | Copy-Item -Destination (Join-Path $destination 'levels')
    Copy-Item -LiteralPath 'assets/fonts/OFL.txt' -Destination (Join-Path $destination 'FONT-LICENSE.txt')
    New-Item -ItemType Directory -Force -Path (Join-Path $destination 'Music') | Out-Null
    Get-ChildItem -LiteralPath 'Music' -File | Copy-Item -Destination (Join-Path $destination 'Music')
    if ($Zip) {
        Compress-Archive -Path "$destination/*" -DestinationPath (Join-Path $projectRoot 'dist/MGame-Overdrive-Windows.zip') -Force
    }
    Write-Output "Ready: $destination/mgame.exe"
} finally {
    $env:RUSTFLAGS = $previousRustFlags
    $env:PATH = $previousPath
    $env:CARGO_HOME = $previousCargoHome
    $env:RUSTUP_HOME = $previousRustupHome
    Pop-Location
}
