; ===== Inno Setup Script for File Organizer =====
; - Installs the release binary produced by cargo
; - Adds Start Menu shortcut + (optional) Desktop
; - "Launch at Windows startup" checkbox (Startup)

#define AppName "File Organizer"
#define AppExe  "file-organizer.exe"
#define AppVersion "2.0.0"
#define Publisher "Unkn0wndo3s"

; SOURCE folder of the binary produced by:
;   cargo build --release --target x86_64-pc-windows-msvc
#define BuildDir "target\x86_64-pc-windows-msvc\release"

[Setup]
AppId={{5A3D9F1B-9C1A-4F4B-9A2B-FA7F9C2DE4E1}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#Publisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableDirPage=no
DisableProgramGroupPage=no
OutputBaseFilename=FileOrganizer-Setup
OutputDir=.
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
WizardStyle=modern
SetupIconFile=fileorganizer.ico

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
; Checkbox: Launch at startup
Name: "startup"; Description: "Launch {#AppName} at Windows startup"; Flags: unchecked
; (Optional) Desktop shortcut
Name: "desktopicon"; Description: "Create a shortcut on the Desktop"; Flags: unchecked

[Files]
; The binary embeds its icons, so only the executable and the tray icon ship
Source: "{#BuildDir}\{#AppExe}"; DestDir: "{app}"
Source: "fileorganizer.ico"; DestDir: "{app}"

[Icons]
; Start Menu
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
; Desktop (optional)
Name: "{commondesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
; Autostart goes through the application itself, which registers the per user
; Run key, so the setting stays in step with the tray menu toggle.
Filename: "{app}\{#AppExe}"; Parameters: "--enable-autostart"; Tasks: startup; \
    Flags: runhidden waituntilterminated
; Launch the app at the end of installation (optional)
Filename: "{app}\{#AppExe}"; Description: "Launch {#AppName} now"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Leave no Run key entry pointing at a binary that is about to disappear
Filename: "{app}\{#AppExe}"; Parameters: "--disable-autostart"; RunOnceId: "DisableAutostart"; \
    Flags: runhidden waituntilterminated
