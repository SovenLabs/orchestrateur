# Convertit une boucle WebM exportée en GIF (nécessite ffmpeg dans le PATH).
param(
    [Parameter(Mandatory = $true)]
    [string]$InputWebm,
    [string]$OutputGif = "",
    [int]$Fps = 15,
    [int]$Width = 1280
)

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Error "ffmpeg introuvable. Installez ffmpeg ou exportez WebM directement depuis l'app."
    exit 1
}

if (-not $OutputGif) {
    $OutputGif = [System.IO.Path]::ChangeExtension($InputWebm, ".gif")
}

$palette = [System.IO.Path]::ChangeExtension($InputWebm, ".palette.png")

ffmpeg -y -i $InputWebm -vf "fps=$Fps,scale=${Width}:-1:flags=lanczos,palettegen" $palette
ffmpeg -y -i $InputWebm -i $palette -lavfi "fps=$Fps,scale=${Width}:-1:flags=lanczos[x];[x][1:v]paletteuse" $OutputGif
Remove-Item $palette -ErrorAction SilentlyContinue

Write-Host "GIF écrit : $OutputGif"