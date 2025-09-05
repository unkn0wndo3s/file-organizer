; ===== Inno Setup Script for File Organizer =====
; - Installe l’app-image jpackage
; - Ajoute un raccourci Menu Démarrer + (optionnel) Bureau
; - Case "Lancer au démarrage de Windows" (Startup)

#define AppName "File Organizer"
#define AppExe  "File Organizer.exe"
#define AppVersion "1.0.0"
#define Publisher "Unkn0wndo3s"

; Dossier SOURCE de l'app-image générée par jpackage
; Exemple: si jpackage a créé .\File Organizer\ avec "File Organizer.exe" dedans :
#define AppImageDir "File Organizer"

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
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
; Case à cocher: Lancer au démarrage
Name: "startup"; Description: "Lancer {#AppName} au démarrage de Windows"; Flags: unchecked
; (Optionnel) Raccourci Bureau
Name: "desktopicon"; Description: "Créer un raccourci sur le Bureau"; Flags: unchecked

[Files]
; Copie TOUT le contenu de l’app-image dans {app}
Source: "{#AppImageDir}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs

[Icons]
; Menu Démarrer
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
; Bureau (optionnel)
Name: "{commondesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon
; Dossier Démarrage (créé seulement si la case 'startup' est cochée)
Name: "{userstartup}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: startup

[Run]
; Lancer l’appli à la fin de l’install (optionnel)
Filename: "{app}\{#AppExe}"; Description: "Lancer {#AppName} maintenant"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Nettoyage du raccourci Startup si présent
Type: files; Name: "{userstartup}\{#AppName}.lnk"
