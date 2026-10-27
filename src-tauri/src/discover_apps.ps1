$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$apps = New-Object 'System.Collections.Generic.List[object]'
$warnings = New-Object 'System.Collections.Generic.List[string]'
$seen = [Collections.Generic.Dictionary[string, bool]]::new([StringComparer]::Ordinal)
$shortcutTargets = @{}

function Add-Candidate($name, $path, $arguments, $workingDirectory) {
    if ([string]::IsNullOrWhiteSpace($path)) { return }
    $path = [Environment]::ExpandEnvironmentVariables($path.Trim().Trim('"'))
    if ($path.IndexOfAny([IO.Path]::GetInvalidPathChars()) -ge 0) { return }
    if ([IO.Path]::GetExtension($path) -ine '.exe') { return }
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return }
    if ($name -match '(?i)uninstall|unins\d*|卸载|解除安装' -or
        [IO.Path]::GetFileName($path) -match '(?i)^(uninstall|unins\d*|msiexec)\.exe$') { return }

    $arguments = [string]$arguments
    $key = $path.ToLowerInvariant() + '|' + $arguments
    if ($seen.ContainsKey($key)) { return }
    $seen[$key] = $true
    if ([string]::IsNullOrWhiteSpace($name)) { $name = [IO.Path]::GetFileNameWithoutExtension($path) }
    if (-not [string]::IsNullOrWhiteSpace($workingDirectory)) {
        $workingDirectory = [Environment]::ExpandEnvironmentVariables($workingDirectory)
    } else {
        $workingDirectory = $null
    }
    $apps.Add([pscustomobject]@{
        name = $name
        path = $path
        args = $arguments
        working_dir = $workingDirectory
    })
}

# Resolve shortcuts without executing or changing them.
$shell = $null
try {
    $shell = New-Object -ComObject WScript.Shell
    $folders = @(
        [Environment]::GetFolderPath('Programs'),
        [Environment]::GetFolderPath('CommonPrograms')
    ) | Select-Object -Unique
    foreach ($folder in $folders) {
        if ([string]::IsNullOrWhiteSpace($folder) -or -not (Test-Path -LiteralPath $folder)) { continue }
        $scanErrors = @()
        $shortcuts = Get-ChildItem -LiteralPath $folder -Filter '*.lnk' -Recurse -File -ErrorAction SilentlyContinue -ErrorVariable scanErrors |
            Sort-Object FullName
        if ($scanErrors.Count -gt 0) { $warnings.Add('Some Start menu folders could not be read.') }
        foreach ($file in $shortcuts) {
            $shortcut = $null
            try {
                $shortcut = $shell.CreateShortcut($file.FullName)
                $before = $apps.Count
                Add-Candidate $file.BaseName $shortcut.TargetPath $shortcut.Arguments $shortcut.WorkingDirectory
                if ($apps.Count -gt $before) {
                    $shortcutTargets[$apps[$apps.Count - 1].path.ToLowerInvariant()] = $true
                }
            } catch {
                $warnings.Add('A Start menu shortcut could not be read.')
            } finally {
                if ($null -ne $shortcut) { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($shortcut) }
            }
        }
    }
} catch {
    $warnings.Add('Start menu discovery is unavailable.')
} finally {
    if ($null -ne $shell) { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($shell) }
}

# App Paths supplement software without Start menu shortcuts, in both registry views.
foreach ($hive in @([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryHive]::LocalMachine)) {
    foreach ($view in @([Microsoft.Win32.RegistryView]::Registry64, [Microsoft.Win32.RegistryView]::Registry32)) {
        $base = $null
        $root = $null
        try {
            $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive, $view)
            $root = $base.OpenSubKey('SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths')
            if ($null -eq $root) { continue }
            foreach ($entry in $root.GetSubKeyNames()) {
                $key = $null
                try {
                    $key = $root.OpenSubKey($entry)
                    if ($null -eq $key) { continue }
                    $path = [string]$key.GetValue('')
                    if ([string]::IsNullOrWhiteSpace($path)) { continue }
                    $path = [Environment]::ExpandEnvironmentVariables($path.Trim().Trim('"'))
                    if ($path.IndexOfAny([IO.Path]::GetInvalidPathChars()) -ge 0) { continue }
                    if ($shortcutTargets.ContainsKey($path.ToLowerInvariant())) { continue }
                    $name = [IO.Path]::GetFileNameWithoutExtension($path)
                    if (Test-Path -LiteralPath $path -PathType Leaf) {
                        $description = [Diagnostics.FileVersionInfo]::GetVersionInfo($path).FileDescription
                        if (-not [string]::IsNullOrWhiteSpace($description)) { $name = $description }
                    }
                    Add-Candidate $name $path '' $null
                } catch {
                    $warnings.Add('An App Paths entry could not be read.')
                } finally {
                    if ($null -ne $key) { $key.Dispose() }
                }
            }
        } catch {
            $warnings.Add('An App Paths registry view could not be read.')
        } finally {
            if ($null -ne $root) { $root.Dispose() }
            if ($null -ne $base) { $base.Dispose() }
        }
    }
}

$result = [pscustomobject]@{
    apps = @($apps.ToArray() | Sort-Object name, path, args)
    warnings = @($warnings.ToArray() | Select-Object -Unique)
}
ConvertTo-Json -InputObject $result -Depth 4 -Compress
