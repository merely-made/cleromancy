[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:TEMP ("cleromancy-headed-" + [DateTime]::UtcNow.ToString("yyyyMMdd-HHmmss"))),
    [string]$TargetDir = "C:\t\cleromancy-headed-target"
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$scenario = Join-Path $PSScriptRoot "headed-tarot.scn"
$store = Join-Path $Root "store"

New-Item -ItemType Directory -Force -Path $Root, $TargetDir | Out-Null
Set-Location $repo

foreach ($phase in "first", "reopen") {
    $captureDir = Join-Path $Root $phase
    New-Item -ItemType Directory -Force -Path $captureDir | Out-Null

    $env:CLEROMANCY_ROOT = $store
    $env:CLEROMANCY_SCENARIO = $scenario
    $env:CLEROMANCY_SCENARIO_PHASE = $phase
    $env:CLEROMANCY_CAPTURE_DIR = $captureDir
    $env:CARGO_TARGET_DIR = $TargetDir
    & cargo run --bin cleromancy --offline
    if ($LASTEXITCODE -ne 0) {
        throw "headed Cleromancy $phase process exited $LASTEXITCODE"
    }

    $done = Join-Path $captureDir "scenario.done"
    $receipt = Join-Path $captureDir "receipt.json"
    $capture = Join-Path $captureDir "detail.png"
    if (!(Test-Path -LiteralPath $done) -or !(Test-Path -LiteralPath $receipt) -or !(Test-Path -LiteralPath $capture)) {
        throw "headed Cleromancy $phase did not leave scenario.done, receipt.json, and detail.png"
    }
    if ((Get-Content -LiteralPath $done -TotalCount 1) -ne "RESULT ok") {
        throw "headed Cleromancy $phase scenario failed: $(Get-Content -LiteralPath $done -Raw)"
    }
}

$first = Get-Content -LiteralPath (Join-Path $Root "first\receipt.json") -Raw | ConvertFrom-Json
$reopen = Get-Content -LiteralPath (Join-Path $Root "reopen\receipt.json") -Raw | ConvertFrom-Json
if (!$first.ok -or !$reopen.ok -or !$first.catalog_ready -or !$reopen.catalog_ready) {
    throw "the headed receipt did not reach a durable catalog state"
}
if ($first.phase -ne "first" -or $reopen.phase -ne "reopen") {
    throw "the headed receipts reported unexpected phases"
}
if ($first.status -notlike "Reflection saved: *" -or $reopen.status -notlike "Viewing session: *") {
    throw "the headed receipts reported unexpected completion states"
}

$idNames = "session_id", "reading_id", "reflection_id"
foreach ($name in $idNames) {
    $firstId = $first.ids.$name
    $reopenId = $reopen.ids.$name
    if ($firstId -notmatch "^[0-9a-f]{64}$" -or $reopenId -notmatch "^[0-9a-f]{64}$") {
        throw "$name is not a typed Cleromancy digest"
    }
    if ($firstId -cne $reopenId) {
        throw "$name changed across the Redb reopen"
    }
}

$redb = Join-Path $store "cleromancy.redb"
if (!(Test-Path -LiteralPath $redb) -or (Get-Item -LiteralPath $redb).Length -le 0) {
    throw "the headed run did not leave a Redb store"
}

$report = [ordered]@{
    schema = "cleromancy.headed-close-reopen/v1"
    ok = $true
    store = $redb
    redb_reopened = $true
    ids = [ordered]@{
        session_id = $first.ids.session_id
        reading_id = $first.ids.reading_id
        reflection_id = $first.ids.reflection_id
    }
    artifacts = [ordered]@{
        first = [ordered]@{
            scenario = (Join-Path $Root "first\scenario.done")
            receipt = (Join-Path $Root "first\receipt.json")
            pixels = (Join-Path $Root "first\detail.png")
        }
        reopen = [ordered]@{
            scenario = (Join-Path $Root "reopen\scenario.done")
            receipt = (Join-Path $Root "reopen\receipt.json")
            pixels = (Join-Path $Root "reopen\detail.png")
        }
    }
    evidence_boundaries = [ordered]@{
        headed_scenario = "semantic Genet interaction, durable worker outcomes, and presented-pixel captures"
        redb_reopen = "a new Cleromancy process opened the same store and selected the matching durable records"
        h0_persistence_replay = "proved separately by cargo test --test headed_consultation --offline"
        windows_keyboard_input = "proved separately by the manual Windows H2 pass; this scenario does not synthesize OS keyboard input"
    }
}
$result = Join-Path $Root "headed-tarot.result.json"
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $result -Encoding utf8
Write-Output "headed Tarot close/reopen receipt: $result"
