# Generate N canonical memory markdown files (UUID v7, tags >=3, backlinks >=3)
param(
    [int]$Count = 100,
    [int]$Seed = 42
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$MemoriesDir = Join-Path $Root "workspace\memories"
New-Item -ItemType Directory -Force -Path $MemoriesDir | Out-Null
Get-ChildItem -Path $MemoriesDir -Filter "*.md" -ErrorAction SilentlyContinue | Remove-Item -Force

$Prefixes = @("Analyse","Note","Synthese","Signal","Pattern","Hypothese","Trace","Fragment","Observation","Modele","Correlation","Projection","Reseau","Noeud","Flux","Boucle","Strategie","Simulation","Artefact","Index")
$Cores = @("neural","cosmique","trading","cortex","daemon","nuance","graphe","embedding","orchestrateur","second-brain","agent","memoire","void","accretion","constellation","insight","backlink","wikilink","phase28","simulation-ia")
$Suffixes = @("alpha","beta","delta","sigma","prime","v2","v3","runtime","live","batch","stream","cluster","hub","lane","pulse","orbit","seed","probe","cache","vector")
$TagPool = @("architecture","cortex","rust","trading","simulation","ia","second-brain","nuance","agent","websocket","godot","memoire","graphe","embedding","daemon","orchestrateur","strategie","cosmic","neural","backlink","wikilink","insight","flux","vector","lancedb")
$Topics = @(
    "correlation activite agent et profondeur de nuance",
    "boucle retroaction memoire chat assimilation",
    "simulation marche appliquee au routing des skills",
    "graphe backlinks pour navigation insights",
    "embedding semantique et seuil de similarite",
    "second brain local souverain sans cloud",
    "trou noir UI comme metaphore de densite informationnelle",
    "constellation agents synchronises sur le daemon",
    "nebuluse memoire et rendu canvas 2D",
    "tickers coherence globale en header"
)

function Format-RfcUuid([byte[]]$bytes) {
    return ('{0:x2}{1:x2}{2:x2}{3:x2}-{4:x2}{5:x2}-{6:x2}{7:x2}-{8:x2}{9:x2}-{10:x2}{11:x2}{12:x2}{13:x2}{14:x2}{15:x2}' -f
        $bytes[0], $bytes[1], $bytes[2], $bytes[3], $bytes[4], $bytes[5], $bytes[6], $bytes[7], $bytes[8], $bytes[9], $bytes[10], $bytes[11], $bytes[12], $bytes[13], $bytes[14], $bytes[15])
}

function New-UuidV7([long]$OffsetMs) {
    $unixMs = [uint64]($OffsetMs -band 0xFFFFFFFFFFFF)
    $rand = New-Object byte[] 10
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($rand)
    $randA = ([uint16]$rand[0] -shl 4) -bor ([uint16]($rand[1] -band 0x0F))
    $bytes = New-Object byte[] 16
    for ($i = 0; $i -lt 6; $i++) { $bytes[$i] = [byte](($unixMs -shr (40 - 8 * $i)) -band 0xFF) }
    $bytes[6] = [byte](0x70 -bor ($randA -shr 8))
    $bytes[7] = [byte]($randA -band 0xFF)
    $bytes[8] = [byte](0x80 -bor ($rand[2] -band 0x3F))
    for ($i = 0; $i -lt 7; $i++) { $bytes[9 + $i] = $rand[3 + $i] }
    # RFC 9562 byte order — ne pas utiliser [Guid]::ToString() (endianness .NET)
    return Format-RfcUuid $bytes
}

$rng = [System.Random]::new($Seed)
$baseMs = [DateTimeOffset]::Parse("2026-06-01T00:00:00Z").ToUnixTimeMilliseconds()
$ids = @()
for ($i = 0; $i -lt $Count; $i++) { $ids += New-UuidV7 ($baseMs + $i * 17) }

$created = 0
for ($i = 0; $i -lt $Count; $i++) {
    $id = $ids[$i]
    $title = "{0} {1} {2}" -f $Prefixes[$rng.Next($Prefixes.Length)], $Cores[$rng.Next($Cores.Length)], $Suffixes[$rng.Next($Suffixes.Length)]
    $tagCount = $rng.Next(3, 7)
    $tags = @()
    while ($tags.Count -lt $tagCount) {
        $pick = $TagPool[$rng.Next($TagPool.Length)]
        if ($pick -notin $tags) { $tags += $pick }
    }
    $tags = $tags | Sort-Object

    $t1 = $ids[($i + 1) % $Count]
    $t2 = $ids[($i + 2) % $Count]
    $t3 = $ids[($i + 3) % $Count]
    $t4 = $ids[($i + 7) % $Count]
    $targets = @($t1, $t2, $t3, $t4) | Select-Object -Unique

    $bl = ""
    $j = 0
    foreach ($target in $targets) {
        $kind = if ($j -eq 0) { "explicit_wikilink" } else { "semantic" }
        $score = if ($kind -eq "explicit_wikilink") { "1.00" } else { ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:F2}", (0.76 + $rng.NextDouble() * 0.22))) }
        $bl += "  - target: `"$target`"`n    score: $score`n    kind: $kind`n"
        $j++
    }

    $tagYaml = ($tags | ForEach-Object { '"' + $_ + '"' }) -join ", "
    $topic = $Topics[$rng.Next($Topics.Length)]
    $wikilinks = ($targets | Select-Object -First 3 | ForEach-Object { "[[$_]]" }) -join " "
    $ts = ([DateTimeOffset]::Parse("2026-06-01T00:00:00Z")).AddMinutes($i * 13).ToString("yyyy-MM-ddTHH:mm:ssZ")
    $safeTitle = $title -replace '"', '\"'
    $tagLines = ($tags | ForEach-Object { "- ``$_``" }) -join "`n"

    $body = @"
# $title

Observation seed **#$($i + 1)** - $topic.

Liens explicites : $wikilinks

## Tags actifs
$tagLines

## Note
Memoire generee pour test de rendu (nebuluse, nodes orbitaux, insights).
"@

    $md = @"
---
id: "$id"
title: "$safeTitle"
tags: [$tagYaml]
created_at: "$ts"
updated_at: "$ts"
backlinks:
$bl---
$body
"@

    $path = Join-Path $MemoriesDir "$id.md"
    [System.IO.File]::WriteAllText($path, $md.TrimEnd() + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))
    $created++
}

Write-Host "OK: $created memories -> $MemoriesDir"
Write-Host "Example: $($ids[0]).md"