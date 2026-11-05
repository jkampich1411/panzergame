; Installer für den Launcher (Inno Setup). Installiert nur für den aktuellen Nutzer, braucht also
; keine Admin-Rechte. Aufruf siehe build.ps1. Der Dateiname enthält keine Version, sonst brechen
; die Links auf der Website.

[Setup]
AppId={{7F3B8E52-4C1D-4A96-9E27-5D0C1B6A8F34}
AppName=Panzer
AppVersion={#AppVersion}
AppPublisher=martinkopoky-wq
AppPublisherURL=https://martinkopoky-wq.github.io/panzergame/
DefaultDirName={localappdata}\Programs\Panzer
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\..\dist
OutputBaseFilename=Panzer-Setup-windows-x86_64
SetupIconFile=..\..\assets\icon\panzer.ico
UninstallDisplayIcon={app}\panzer-launcher.exe
UninstallDisplayName=Panzer
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\panzer-launcher.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Panzer"; Filename: "{app}\panzer-launcher.exe"
Name: "{autodesktop}\Panzer"; Filename: "{app}\panzer-launcher.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\panzer-launcher.exe"; Description: "{cm:LaunchProgram,Panzer}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Das vom Launcher heruntergeladene Spiel (siehe data_dir in crates/launcher/src/main.rs)
Type: filesandordirs; Name: "{localappdata}\Panzer"
