# Generate app.ico (32x32) for Windows installer.
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$size = 32
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.Clear([System.Drawing.Color]::FromArgb(255, 18, 18, 32))

$brushOuter = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 40, 80, 140))
$brushInner = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 74, 158, 255))
$cx = 15.5
$cy = 15.5
$g.FillEllipse($brushOuter, $cx - 11, $cy - 11, 22, 22)
$g.FillEllipse($brushInner, $cx - 9, $cy - 9, 18, 18)

$icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
$fs = [System.IO.File]::Open($OutputPath, [System.IO.FileMode]::Create)
$icon.Save($fs)
$fs.Close()

$g.Dispose()
$bmp.Dispose()
$icon.Dispose()
$brushOuter.Dispose()
$brushInner.Dispose()

Write-Host "Icon created: $OutputPath"