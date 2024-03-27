$InstallDir = New-Item -Path $env:LOCALAPPDATA -Name ArctisBatteryIndicator -ItemType Directory -Force

# Kill previous process if running
if (Get-Process "arctis-battery-indicator" -ErrorAction SilentlyContinue) {
  $id = (Get-Process "arctis-battery-indicator").Id
  Stop-Process $id
  Wait-Process $id -ErrorAction SilentlyContinue

  Start-Sleep 0.5
}

Remove-Item "$InstallDir\*" -Exclude .log

Write-Output "Removed files from previous version"

foreach ($exe in @("arctis-battery-indicator.exe", "arctis-battery-indicator-debug.exe")) {
  Copy-Item $exe "$InstallDir\$exe"
}

Write-Output "Copied executables to $InstallDir"

$ServiceExePath = "$InstallDir\arctis-battery-indicator.exe"

$WshShell = New-Object -ComObject WScript.Shell
$ShortcutPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\arctis-battery-indicator.lnk"

$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $ServiceExePath
$Shortcut.Save()

Write-Output "Added shortcut to $ShortcutPath"

Start-Process $ServiceExePath -WorkingDirectory $InstallDir

Write-Output "Started Arctis Battery Indicator"

Write-Output "Installation complete!"

Pause