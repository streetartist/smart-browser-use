param(
    [string]$ServerName = "smart-browser-use",
    [string]$CodexHome = "$env:USERPROFILE\.codex",
    [string]$TargetDir = "$env:USERPROFILE\.codex\memories\smart-browser-use-target",
    [switch]$Headless,
    [switch]$NoBuild,
    [switch]$StopRunning
)

$ErrorActionPreference = "Stop"

function Info($Message) {
    Write-Host "[smart-browser-use mcp] $Message"
}

function Quote-TomlLiteral($Value) {
    return "'" + ($Value -replace "'", "''") + "'"
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$ConfigPath = Join-Path $CodexHome "config.toml"

if (-not (Test-Path $CodexHome)) {
    New-Item -ItemType Directory -Path $CodexHome | Out-Null
}

if (-not (Test-Path $ConfigPath)) {
    New-Item -ItemType File -Path $ConfigPath | Out-Null
}

if (-not $NoBuild) {
    Info "building mcp-server.exe"
    Push-Location $RepoRoot
    try {
        $env:CARGO_INCREMENTAL = "0"
        cargo build --features mcp-server --bin mcp-server --target-dir $TargetDir
    }
    finally {
        Pop-Location
    }
}

$ExePath = Join-Path $TargetDir "debug\mcp-server.exe"
if (-not (Test-Path $ExePath)) {
    throw "mcp-server.exe not found at $ExePath. Run without -NoBuild or check the build output."
}

if ($StopRunning) {
    Info "stopping running mcp-server processes for this executable"
    Get-Process mcp-server -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            if ($_.Path -eq $ExePath) {
                Stop-Process -Id $_.Id -Force
            }
        }
        catch {
            # Some process entries may not expose Path; ignore them.
        }
    }
}

$BackupPath = "$ConfigPath.bak-smart-browser-use-$(Get-Date -Format yyyyMMddHHmmss)"
Copy-Item -Path $ConfigPath -Destination $BackupPath -Force
Info "backed up config to $BackupPath"

$Text = Get-Content -Path $ConfigPath -Raw
if ($null -eq $Text) {
    $Text = ""
}

$EscapedName = [regex]::Escape($ServerName)
$SectionPattern = "(?ms)^\[mcp_servers\.$EscapedName\]\r?\n.*?(?=^\[|\z)"
$Text = [regex]::Replace($Text, $SectionPattern, "")
$Text = $Text.TrimEnd()

$Args = @()
if (-not $Headless) {
    $Args += "--headed"
}

$ArgsToml = ($Args | ForEach-Object { Quote-TomlLiteral $_ }) -join ", "
$ExeToml = Quote-TomlLiteral $ExePath

$Block = @"

[mcp_servers.$ServerName]
command = $ExeToml
args = [$ArgsToml]
"@

$NewText = $Text + $Block + "`r`n"
[System.IO.File]::WriteAllText($ConfigPath, $NewText, [System.Text.UTF8Encoding]::new($false))

Info "installed MCP server '$ServerName'"
Info "command: $ExePath"
Info "args: $($Args -join ' ')"
Info "restart Codex to load the new MCP server"
