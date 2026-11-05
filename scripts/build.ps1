param([string]$Toolchain = '', [switch]$Check)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
$previousRustFlags = $env:RUSTFLAGS
try {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    $cargoExe = if ($cargoCommand) { $cargoCommand.Source } else { Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe' }
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
    New-Item -ItemType Directory -Force -Path (Join-Path $destination 'Maps') | Out-Null
    Get-ChildItem -LiteralPath 'Maps' -File | Copy-Item -Destination (Join-Path $destination 'Maps')
    New-Item -ItemType Directory -Force -Path (Join-Path $destination 'levels') | Out-Null
    Get-ChildItem -LiteralPath 'levels' -File | Copy-Item -Destination (Join-Path $destination 'levels')
    Copy-Item -LiteralPath 'assets/fonts/OFL.txt' -Destination (Join-Path $destination 'FONT-LICENSE.txt')
    Compress-Archive -Path "$destination/*" -DestinationPath (Join-Path $projectRoot 'dist/MGame-Overdrive-Windows.zip') -Force
    Write-Output "Ready: $destination/mgame.exe"
} finally {
    $env:RUSTFLAGS = $previousRustFlags
    Pop-Location
}
