param(
    [Parameter(Mandatory = $true)]
    [string]$PackageDirectory
)

$ErrorActionPreference = 'Stop'
$installDirectory = Join-Path $env:LOCALAPPDATA 'Programs\CommandScheduler'
$daemon = Join-Path $PackageDirectory 'command-scheduler.exe'
$cli = Join-Path $PackageDirectory 'command-scheduler-cli.exe'
$webDirectory = Join-Path $PackageDirectory 'ui'
if (-not (Test-Path $daemon -PathType Leaf)) { throw "Missing daemon: $daemon" }
if (-not (Test-Path $cli -PathType Leaf)) { throw "Missing CLI: $cli" }
if (-not (Test-Path (Join-Path $webDirectory 'index.html') -PathType Leaf)) { throw "Missing UI assets: $webDirectory" }

$taskName = 'Command Scheduler'
$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($existingTask -and $existingTask.State -eq 'Running') {
    Stop-ScheduledTask -TaskName $taskName
    $stopDeadline = (Get-Date).AddSeconds(15)
    do {
        Start-Sleep -Milliseconds 250
        $existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    } while (
        $existingTask -and
        $existingTask.State -eq 'Running' -and
        (Get-Date) -lt $stopDeadline
    )
    if ($existingTask -and $existingTask.State -eq 'Running') {
        throw "The existing daemon task did not stop within 15 seconds."
    }
}

New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null
Copy-Item $daemon (Join-Path $installDirectory 'command-scheduler.exe') -Force
Copy-Item $cli (Join-Path $installDirectory 'command-scheduler-cli.exe') -Force
if (Test-Path (Join-Path $installDirectory 'ui')) { Remove-Item (Join-Path $installDirectory 'ui') -Recurse -Force }
Copy-Item $webDirectory (Join-Path $installDirectory 'ui') -Recurse -Force

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathEntries = @($userPath -split ';' | Where-Object { $_ })
if ($pathEntries -notcontains $installDirectory) {
    [Environment]::SetEnvironmentVariable('Path', (($pathEntries + $installDirectory) -join ';'), 'User')
}

$action = New-ScheduledTaskAction -Execute (Join-Path $installDirectory 'command-scheduler.exe') -WorkingDirectory $installDirectory
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero)
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Settings $settings -Description 'Local Command Scheduler daemon' -Force | Out-Null
Start-ScheduledTask -TaskName $taskName
Write-Output "Installed in $installDirectory and started for user $env:USERNAME. Open a new terminal to use command-scheduler-cli."
