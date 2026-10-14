$ErrorActionPreference = 'Stop'
$tokens = $null
$errors = $null
$installer = Join-Path $PSScriptRoot '../../packaging/windows/install-service.ps1'
$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path $installer), [ref]$tokens, [ref]$errors
)
if ($errors.Count -gt 0) { throw ($errors | Out-String) }
$commands = @($ast.FindAll({
    param($node)
    $node -is [System.Management.Automation.Language.CommandAst] -and
    $node.GetCommandName() -eq 'New-ScheduledTaskSettingsSet'
}, $true))
if ($commands.Count -ne 1) { throw 'Expected exactly one installer task settings command.' }
# Execute only the settings constructor from the installer, never its installation.
$settings = & ([scriptblock]::Create($commands[0].Extent.Text))
if ($settings.ExecutionTimeLimit -ne 'PT0S') { throw 'The daemon task must have an unlimited execution duration.' }
$validationTaskName = 'CommandScheduler-validation-' + [guid]::NewGuid().ToString()
try {
    $action = New-ScheduledTaskAction -Execute (Join-Path $env:SystemRoot 'System32/cmd.exe') -Argument '/c exit 0'
    Register-ScheduledTask -TaskName $validationTaskName -Action $action -Settings $settings | Out-Null
    $registered = Get-ScheduledTask -TaskName $validationTaskName
    if ($registered.Settings.ExecutionTimeLimit -ne 'PT0S') { throw 'Registered task reintroduced an execution time limit.' }
} finally {
    Unregister-ScheduledTask -TaskName $validationTaskName -Confirm:$false -ErrorAction SilentlyContinue
}
