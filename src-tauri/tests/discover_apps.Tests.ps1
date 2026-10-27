param([string]$DiscoveryScriptPath)
$ErrorActionPreference = 'Stop'

# Test candidate filtering without scanning or launching the computer's software.
if (-not $DiscoveryScriptPath) { $DiscoveryScriptPath = Join-Path $PSScriptRoot '../src/discover_apps.ps1' }
$source = Get-Content $DiscoveryScriptPath -Raw -Encoding UTF8
$ast = [scriptblock]::Create($source).Ast
$definition = $ast.Find({
    param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Add-Candidate'
}, $true)
. ([scriptblock]::Create($definition.Extent.Text))
$apps = New-Object 'System.Collections.Generic.List[object]'
$seen = [Collections.Generic.Dictionary[string, bool]]::new([StringComparer]::Ordinal)
$executable = Join-Path $env:SystemRoot 'System32/cmd.exe'
$workingDirectory = [IO.Path]::GetDirectoryName($executable)
$arguments = '--profile "工作 配置"'

function Assert($condition, $description) {
    if (-not $condition) { throw $description }
}

Add-Candidate '中文软件' $executable $arguments $workingDirectory
Assert ($apps.Count -eq 1) 'A valid candidate should be included.'
Assert ($apps[0].name -eq '中文软件') 'Unicode names must survive discovery.'
Assert ($apps[0].args -ceq $arguments) 'Quoted arguments must be preserved exactly.'
Assert ($apps[0].working_dir -eq $workingDirectory) 'Working directory must be preserved.'

Add-Candidate 'Duplicate' $executable.ToUpperInvariant() $arguments $null
Assert ($apps.Count -eq 1) 'Windows executable paths must be deduplicated without case sensitivity.'
Add-Candidate 'Different profile' $executable '--profile other' $null
Assert ($apps.Count -eq 2) 'Different shortcut arguments must remain separate launch choices.'
Assert ($null -eq $apps[1].working_dir) 'Absent working directories must serialize as null.'

Add-Candidate '卸载软件' $executable '--uninstall' $null
Add-Candidate 'Uninstall Example' $executable '--remove' $null
Add-Candidate 'Not executable' (Join-Path $env:SystemRoot 'System32/shell32.dll') '' $null
Add-Candidate 'Missing file' (Join-Path $env:TEMP 'sofsta-nonexistent/app.exe') '' $null
Add-Candidate 'Empty path' '' '' $null
Add-Candidate 'Malformed path' '"C:\Example.exe" --argument' '' $null
Assert ($apps.Count -eq 2) 'Uninstall entries, non-executable targets, and invalid paths must be excluded.'

Add-Candidate '' '"%SystemRoot%\System32\cmd.exe"' '' $null
Assert ($apps.Count -eq 3) 'Quoted paths and environment variables must be resolved.'
Assert ($apps[2].name -eq 'cmd') 'A missing display name should fall back to the executable name.'

Add-Candidate 'Case-sensitive profile' $executable '--profile OTHER' $null
Assert ($apps.Count -eq 4) 'Case-sensitive argument values must remain separate launch choices.'

Write-Output 'PASS: discovery filtering, deduplication, Unicode names, parameters, and working directories.'
