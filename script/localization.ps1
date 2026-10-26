$ErrorActionPreference = "Stop"

$Python = $env:PYTHON
if (-not $Python) {
    if (Get-Command "python" -ErrorAction SilentlyContinue) {
        $Python = "python"
    } elseif (Get-Command "python3" -ErrorAction SilentlyContinue) {
        $Python = "python3"
    } elseif (Get-Command "py" -ErrorAction SilentlyContinue) {
        $Python = "py"
    } else {
        throw "Python 3 was not found on PATH. Set the PYTHON environment variable to its executable."
    }
}

& $Python (Join-Path $PSScriptRoot "localization") @args
exit $LASTEXITCODE
