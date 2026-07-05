Add-Type -AssemblyName System.Drawing
$size = 128
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.Clear([System.Drawing.Color]::FromArgb(255, 10, 10, 14))

$violet = [System.Drawing.Color]::FromArgb(255, 124, 111, 251)
$ink    = [System.Drawing.Color]::FromArgb(255, 214, 214, 227)

$scale = 2
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$path.AddBezier(10*$scale,44*$scale, 22*$scale,44*$scale, 20*$scale,20*$scale, 32*$scale,20*$scale)
$path.AddBezier(32*$scale,20*$scale, 44*$scale,20*$scale, 42*$scale,44*$scale, 54*$scale,44*$scale)

$penWidth = [float](5*$scale)
$pen = New-Object System.Drawing.Pen($violet, $penWidth)
$pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$g.DrawPath($pen, $path)

$r = 5 * $scale
$brushV = New-Object System.Drawing.SolidBrush($violet)
$brushI = New-Object System.Drawing.SolidBrush($ink)

$g.FillEllipse($brushV, (10*$scale - $r), (44*$scale - $r), $r*2, $r*2)
$g.FillEllipse($brushI, (32*$scale - $r), (20*$scale - $r), $r*2, $r*2)
$g.FillEllipse($brushV, (54*$scale - $r), (44*$scale - $r), $r*2, $r*2)

$out = "D:\Claude-Workspace\thruline\packages\vscode\media\logo-128.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose(); $pen.Dispose(); $brushV.Dispose(); $brushI.Dispose()
"saved: $out"
