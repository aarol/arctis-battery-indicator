; Heavily inspired by Ollama: https://github.com/ollama/ollama/blob/main/app/ollama.iss

#define MyAppExeName "arctis-battery-indicator.exe"
#define MyAppDebugExeName "arctis-battery-indicator-debug.exe"
#define MyAppDisplayName "Arctis Battery Indicator"

[Setup]
AppId=88ECD258-57B9-4DDB-ABA3-67DC0289A92C
AppName={#MyAppDisplayName}
; keep this up to date
AppVersion=2.1.1
WizardStyle=modern
DefaultDirName={localappdata}\Programs\ArctisBatteryIndicator
DefaultGroupName={#MyAppDisplayName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Since no icons will be created in "{group}", we don't need the wizard
; to ask for a Start Menu folder name:
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
SetupIconFile=src/bat/main.ico
Compression=lzma2
SolidCompression=yes
OutputBaseFilename="ArctisBatteryIndicatorSetup"
PrivilegesRequired=lowest
CloseApplications=force
RestartApplications=no
DirExistsWarning=no

SetupMutex=Arctis-Indicator-Mutex

[Files]
Source: "target/release/arctis-battery-indicator.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "target/release/arctis-battery-indicator-debug.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "example.config.toml"; DestDir: "{app}"

[Icons]
Name: "{userstartup}\Arctis Battery Indicator"; Filename: "{app}\arctis-battery-indicator.exe";

[Run]
Filename: "{app}\arctis-battery-indicator.exe"; WorkingDir: "{app}"; Description: "Launch application (it will show up inside the task bar ^-arrow menu)"; Flags: postinstall nowait

[InstallDelete]
Type: filesandordirs; Name: "{%LOCALAPPDATA}\ArctisBatteryIndicator"
Type: filesandordirs; Name: "{%LOCALAPPDATA}\Programs\ArctisBatteryIndicator"
Type: filesandordirs; Name: "{userstartup}\arctis-battery-indicator.lnk"

[UninstallRun]
Filename: "taskkill"; Parameters: "/im ""arctis-battery-indicator.exe"" /f /t"; Flags: runhidden; RunOnceId: "Kill exe"
Filename: "taskkill"; Parameters: "/im ""arctis-battery-indicator-debug.exe"" /f /t"; Flags: runhidden; RunOnceId: "Kill exe (debug)"

; HACK!  need to give the server and app enough time to exit
Filename: "{cmd}"; Parameters: "/c timeout 1"; Flags: runhidden; RunOnceId: "Wait"

[UninstallDelete]
Type: files; Name: "{app}\arctis-battery-indicator.log"
Type: dirifempty; Name: "{app}"

[Code]
function KillProcessByName(ProcessName: string): Boolean;
var
  ResultCode: Integer;
begin
  // Attempt to kill a process by name using taskkill
  Result := Exec('taskkill.exe', '/F /IM "' + ProcessName + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

// Runs after the user has pressed "install"
// Why is it necessary to kill the previous process with taskkill instead of relying on 
// the Windows restart manager that Inno Setup normally uses?
// 
// The tray-icon crate creates a new window via the win32 api, but actually doesn't
// provide a way to react to the events sent by restart manager (WM_QUERYSESSIONEND, WM_CLOSE)
// this means that there's currently no way to gracefully close the program, so we have to manually kill it.
function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  KillProcessByName('arctis-battery-indicator.exe')
  Result := '';  // Empty string means continue installation
end;
