[Setup]
AppId={{1C4B88E5-19B0-4B6B-9C41-1E4C5B3B7B91}
AppName=FileOrganizer
AppVersion=1.0
AppPublisher=unkn0wndo3s
DefaultDirName={autopf}\FileOrganizer
DefaultGroupName=FileOrganizer
OutputDir=Output
OutputBaseFilename=FileOrganizer-Setup
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
DisableProgramGroupPage=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "startup"; Description: "Launch at Windows startup"; Flags: unchecked
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "dist\FileOrganizer\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{group}\FileOrganizer"; Filename: "{app}\bin\FileOrganizer.exe"
Name: "{commondesktop}\FileOrganizer"; Filename: "{app}\bin\FileOrganizer.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\FileOrganizer.exe"; Description: "Run FileOrganizer now"; Flags: nowait postinstall skipifsilent

[Registry]
; Per-user startup (si install per-user)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; \
  ValueName: "FileOrganizer"; ValueData: """{app}\bin\FileOrganizer.exe"""; \
  Tasks: startup; Check: not IsAdminInstallMode

; Machine-wide startup (si install tous les utilisateurs)
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; \
  ValueName: "FileOrganizer"; ValueData: """{app}\bin\FileOrganizer.exe"""; \
  Tasks: startup; Check: IsAdminInstallMode

; Cleanup à la désinstallation
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; \
  ValueName: "FileOrganizer"; Flags: deletevalue uninsdeletevalue
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; \
  ValueName: "FileOrganizer"; Flags: deletevalue uninsdeletevalue
