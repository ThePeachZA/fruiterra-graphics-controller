param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('show', 'hide', 'force_update', 'set_scene', 'next_scene', 'previous_scene')]
    [string]$Action,

    [string]$Scene,

    [ValidateRange(1, 65535)]
    [int]$Port = 3001
)

$ErrorActionPreference = 'Stop'

if ($Action -eq 'set_scene' -and [string]::IsNullOrWhiteSpace($Scene)) {
    throw "set_scene requires -Scene <file>."
}

if ($PSBoundParameters.ContainsKey('Scene') -and $Scene -notmatch '^[A-Za-z0-9._-]{1,128}$') {
    throw "The scene name contains unsupported characters."
}

$configuredRoot = $env:FGC_DATA_DIR
if (-not [string]::IsNullOrWhiteSpace($configuredRoot)) {
    $tokenPath = Join-Path $configuredRoot 'data\control-token'
}
else {
    $appData = $env:APPDATA
    if ([string]::IsNullOrWhiteSpace($appData)) {
        throw "APPDATA is not set and FGC_DATA_DIR was not provided."
    }

    $releaseToken = Join-Path $appData 'Fruiterra\Fruiterra Graphics Controller\data\control-token'
    $debugToken = Join-Path $appData 'Fruiterra\Fruiterra Graphics Controller (dev)\data\control-token'

    if (Test-Path -LiteralPath $debugToken -PathType Leaf) {
        $tokenPath = $debugToken
    }
    else {
        $tokenPath = $releaseToken
    }
}

if (-not (Test-Path -LiteralPath $tokenPath -PathType Leaf)) {
    throw "Control token not found at $tokenPath. Open Settings -> Local control in the app."
}

$token = (Get-Content -LiteralPath $tokenPath -Raw).Trim()
if ($token -notmatch '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$') {
    throw "The control token is not a valid UUIDv4 string. Regenerate it in Settings -> Local control."
}

$timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
$nonce = '{0}-{1}-{2}' -f $timestamp, $PID, [guid]::NewGuid().ToString('N')

$payload = [ordered]@{
    action    = $Action
    timestamp = $timestamp
    nonce     = $nonce
}
if ($Action -eq 'set_scene') {
    $payload.scene = $Scene
}

$body = $payload | ConvertTo-Json -Compress -Depth 2
$endpoint = "http://127.0.0.1:$Port/control"

$response = Invoke-RestMethod -Method Post -Uri $endpoint `
    -Headers @{ Authorization = "Bearer $token" } `
    -ContentType 'application/json' -Body $body -TimeoutSec 5

if (-not $response -or -not $response.ok) {
    throw "Control request was not accepted."
}

Write-Output ("{0}: ok" -f $Action)
