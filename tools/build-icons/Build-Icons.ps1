<#
.SYNOPSIS
    Export placeholder-backed app icon assets for each OS.

.DESCRIPTION
    Renders assets/brand/source/icon.svg into deterministic per-OS icon assets:
      - PNG sizes for Linux/generic consumers
      - Windows .ico plus PNG intermediates
      - macOS .icns plus PNG intermediates

    This is a pipeline-only script. The default SVG is intentionally a
    placeholder with TODOs; do not treat generated output as final brand art.

.PARAMETER Source
    Source SVG path. Defaults to assets/brand/source/icon.svg.

.PARAMETER OutDir
    Generated output root. Defaults to assets/brand/generated/icons.
#>
[CmdletBinding()]
param(
    [string]$Source = (Join-Path $PSScriptRoot '..\..\assets\brand\source\icon.svg'),
    [string]$OutDir = (Join-Path $PSScriptRoot '..\..\assets\brand\generated\icons')
)

$ErrorActionPreference = 'Stop'

function Resolve-OrCreateDirectory([string]$Path) {
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
    return (Resolve-Path $Path).Path
}

function Find-Tool([string]$Name) {
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $cargoExe = Join-Path $HOME ".cargo\bin\$Name.exe"
    if (Test-Path $cargoExe) {
        return $cargoExe
    }

    $cargoBin = Join-Path $HOME ".cargo\bin\$Name"
    if (Test-Path $cargoBin) {
        return $cargoBin
    }

    return $null
}

function Convert-SvgToPng([string]$Svg, [string]$Png, [int]$Size) {
    if ($script:Resvg) {
        & $script:Resvg -w $Size -h $Size $Svg $Png
        return
    }

    if ($script:RsvgConvert) {
        & $script:RsvgConvert -w $Size -h $Size $Svg -o $Png
        return
    }

    if ($script:Magick) {
        & $script:Magick -background none -density 384 $Svg -resize "${Size}x${Size}" $Png
        return
    }

    if ($script:Python) {
        & $script:Python -c "import cairosvg; cairosvg.svg2png(url=r'$Svg', write_to=r'$Png', output_width=$Size, output_height=$Size)"
        return
    }

    throw 'No SVG renderer found. Install resvg, rsvg-convert, ImageMagick, or python+cairosvg.'
}

function New-Ico([string[]]$Pngs, [string]$IcoPath) {
    if ($script:Magick) {
        & $script:Magick $Pngs $IcoPath
        return
    }

    if ($script:Python) {
        $pngList = ($Pngs | ForEach-Object { "r'$_'" }) -join ','
        & $script:Python -c @"
from PIL import Image
pngs = [$pngList]
images = [Image.open(path).convert('RGBA') for path in pngs]
images[-1].save(r'$IcoPath', sizes=[image.size for image in images])
"@
        return
    }

    throw 'No ICO assembler found. Install ImageMagick or python+Pillow.'
}

function New-Icns([hashtable]$PngBySize, [string]$IcnsPath) {
    if ($IsMacOS) {
        $iconset = Join-Path ([System.IO.Path]::GetDirectoryName($IcnsPath)) 'icon.iconset'
        Resolve-OrCreateDirectory $iconset | Out-Null

        Copy-Item $PngBySize[16] (Join-Path $iconset 'icon_16x16.png') -Force
        Copy-Item $PngBySize[32] (Join-Path $iconset 'icon_16x16@2x.png') -Force
        Copy-Item $PngBySize[32] (Join-Path $iconset 'icon_32x32.png') -Force
        Copy-Item $PngBySize[64] (Join-Path $iconset 'icon_32x32@2x.png') -Force
        Copy-Item $PngBySize[128] (Join-Path $iconset 'icon_128x128.png') -Force
        Copy-Item $PngBySize[256] (Join-Path $iconset 'icon_128x128@2x.png') -Force
        Copy-Item $PngBySize[256] (Join-Path $iconset 'icon_256x256.png') -Force
        Copy-Item $PngBySize[512] (Join-Path $iconset 'icon_256x256@2x.png') -Force
        Copy-Item $PngBySize[512] (Join-Path $iconset 'icon_512x512.png') -Force
        Copy-Item $PngBySize[1024] (Join-Path $iconset 'icon_512x512@2x.png') -Force

        iconutil -c icns $iconset -o $IcnsPath
        Remove-Item $iconset -Recurse -Force
        return
    }

    if ($script:Python) {
        & $script:Python -c @"
import struct
from pathlib import Path

chunks = [
    (16, 'icp4'),
    (32, 'icp5'),
    (64, 'icp6'),
    (128, 'ic07'),
    (256, 'ic08'),
    (512, 'ic09'),
    (1024, 'ic10'),
]
paths = {
    16: r'$($PngBySize[16])',
    32: r'$($PngBySize[32])',
    64: r'$($PngBySize[64])',
    128: r'$($PngBySize[128])',
    256: r'$($PngBySize[256])',
    512: r'$($PngBySize[512])',
    1024: r'$($PngBySize[1024])',
}
payload = bytearray()
for size, code in chunks:
    data = Path(paths[size]).read_bytes()
    payload += code.encode('ascii') + struct.pack('>I', len(data) + 8) + data
Path(r'$IcnsPath').write_bytes(b'icns' + struct.pack('>I', len(payload) + 8) + payload)
"@
        return
    }

    throw 'No ICNS assembler found. Install python, or run on macOS with iconutil.'
}

$sourcePath = (Resolve-Path $Source).Path
$outputRoot = Resolve-OrCreateDirectory $OutDir
$linuxDir = Resolve-OrCreateDirectory (Join-Path $outputRoot 'png')
$windowsDir = Resolve-OrCreateDirectory (Join-Path $outputRoot 'windows')
$macosDir = Resolve-OrCreateDirectory (Join-Path $outputRoot 'macos')

$script:Resvg = Find-Tool 'resvg'
$script:RsvgConvert = Find-Tool 'rsvg-convert'
$script:Magick = Find-Tool 'magick'
$script:Python = Find-Tool 'python'

$renderer = @($script:Resvg, $script:RsvgConvert, $script:Magick) |
    Where-Object { $_ } |
    Select-Object -First 1
if (-not $renderer -and $script:Python) {
    $renderer = 'python+cairosvg'
}

Write-Host "Source      : $sourcePath"
Write-Host "Output root : $outputRoot"
Write-Host "Renderer    : $renderer"
Write-Host 'Notice      : placeholder source only; replace the SVG before release.'

$sizes = @(16, 32, 48, 64, 128, 256, 512, 1024)
$pngBySize = @{}

foreach ($size in $sizes) {
    $png = Join-Path $linuxDir "icon-$size.png"
    Convert-SvgToPng $sourcePath $png $size
    $pngBySize[$size] = $png
    Write-Host "PNG         : icon-$size.png"
}

foreach ($size in @(16, 32, 48, 64, 128, 256, 512)) {
    Copy-Item $pngBySize[$size] (Join-Path $windowsDir "icon-$size.png") -Force
}
New-Ico @($pngBySize[16], $pngBySize[32], $pngBySize[48], $pngBySize[256]) (Join-Path $windowsDir 'app.ico')
Write-Host 'Windows ICO : windows/app.ico'

foreach ($size in @(16, 32, 64, 128, 256, 512, 1024)) {
    Copy-Item $pngBySize[$size] (Join-Path $macosDir "icon-$size.png") -Force
}
New-Icns $pngBySize (Join-Path $macosDir 'app.icns')
Write-Host 'macOS ICNS  : macos/app.icns'

Write-Host 'Done.'
