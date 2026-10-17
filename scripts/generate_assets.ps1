Add-Type -AssemblyName System.Drawing

$assetsDir = Join-Path $PSScriptRoot "..\assets"
if (-not (Test-Path $assetsDir)) {
    New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
}

$bmp = [System.Drawing.Bitmap]::new(256, 256)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

# Background rounded rect with deep dark slate
$path = [System.Drawing.Drawing2D.GraphicsPath]::new()
$radius = 48
$path.AddArc(16, 16, $radius, $radius, 180, 90)
$path.AddArc(16 + 224 - $radius, 16, $radius, $radius, 270, 90)
$path.AddArc(16 + 224 - $radius, 16 + 224 - $radius, $radius, $radius, 0, 90)
$path.AddArc(16, 16 + 224 - $radius, $radius, $radius, 90, 90)
$path.CloseFigure()

# Solid / Gradient fill
$pt1 = [System.Drawing.Point]::new(16, 16)
$pt2 = [System.Drawing.Point]::new(240, 240)
$c1 = [System.Drawing.Color]::FromArgb(255, 30, 41, 59)
$c2 = [System.Drawing.Color]::FromArgb(255, 15, 23, 42)
$brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($pt1, $pt2, $c1, $c2)
$g.FillPath($brush, $path)

# Border cyan glow
$penGlow = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(200, 56, 189, 248), 4)
$g.DrawPath($penGlow, $path)

# Reticle / Crop corner brackets
$penReticle = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(255, 56, 189, 248), 8)
$penReticle.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$penReticle.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

# Top-Left corner
$g.DrawLine($penReticle, 60, 85, 60, 60)
$g.DrawLine($penReticle, 60, 60, 85, 60)
# Top-Right corner
$g.DrawLine($penReticle, 196, 85, 196, 60)
$g.DrawLine($penReticle, 196, 60, 171, 60)
# Bottom-Left corner
$g.DrawLine($penReticle, 60, 171, 60, 196)
$g.DrawLine($penReticle, 60, 196, 85, 196)
# Bottom-Right corner
$g.DrawLine($penReticle, 196, 171, 196, 196)
$g.DrawLine($penReticle, 196, 196, 171, 196)

# Central Electric Lightning Bolt
$pts = [System.Drawing.Point[]]@(
    [System.Drawing.Point]::new(140, 65),
    [System.Drawing.Point]::new(95, 130),
    [System.Drawing.Point]::new(125, 130),
    [System.Drawing.Point]::new(115, 190),
    [System.Drawing.Point]::new(165, 120),
    [System.Drawing.Point]::new(135, 120)
)
$bpt1 = [System.Drawing.Point]::new(95, 65)
$bpt2 = [System.Drawing.Point]::new(165, 190)
$bc1 = [System.Drawing.Color]::FromArgb(255, 250, 204, 21)
$bc2 = [System.Drawing.Color]::FromArgb(255, 249, 115, 22)
$boltBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($bpt1, $bpt2, $bc1, $bc2)
$g.FillPolygon($boltBrush, $pts)

$pngPath = Join-Path $assetsDir "icon.png"
$bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Generate proper Multi-Resolution ICO
$icoPath = Join-Path $assetsDir "icon.ico"
$hIcon = $bmp.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hIcon)
$fs = [System.IO.FileStream]::new($icoPath, [System.IO.FileMode]::Create)
$icon.Save($fs)
$fs.Close()

Write-Host "Created $pngPath and $icoPath successfully!"
