; Inno Setup 6 — Orchestrateur Windows installer
; Compilé via scripts/build-installer.ps1 (ISCC + defines StagingRoot / MyAppVersion)

#ifndef StagingRoot
  #define StagingRoot "..\dist\staging"
#endif

#ifndef MyAppVersion
  #define MyAppVersion "0.5.0"
#endif

#ifndef MyAppVersionFull
  #define MyAppVersionFull "0.5.0.0"
#endif

#define MyAppName "Orchestrateur"
#define MyAppPublisher "Sovën"
#define MyAppURL "https://github.com/SovenLabs/orchestrateur"
#define MyAppExeCLI "orchestrateur.exe"
#define MyAppExeHUD "orchestrateur-hud.exe"

[Setup]
AppId={{8F2A1B3C-4D5E-6F70-8A9B-0C1D2E3F4A5B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile={#StagingRoot}\LICENSE.txt
InfoBeforeFile={#StagingRoot}\INSTALL.txt
OutputDir=..\dist
OutputBaseFilename=Orchestrateur-v{#MyAppVersion}-Setup-win64
SetupIconFile={#StagingRoot}\app.ico
UninstallDisplayIcon={app}\{#MyAppExeHUD}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
VersionInfoVersion={#MyAppVersionFull}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} — second cerveau local souverain
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le Bureau"; GroupDescription: "Raccourcis:"; Flags: unchecked

[Files]
Source: "{#StagingRoot}\orchestrateur.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\orchestrateur-hud.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\workspace\*"; DestDir: "{app}\workspace"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#StagingRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\INSTALL.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\app.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Orchestrateur (interface graphique)"; Filename: "{app}\{#MyAppExeHUD}"; Parameters: "--workspace ""{code:GetWorkspaceRoot}"""; WorkingDir: "{app}"
Name: "{group}\Orchestrateur (terminal / TUI)"; Filename: "{app}\{#MyAppExeCLI}"; Parameters: "--workspace ""{code:GetWorkspaceRoot}"""; WorkingDir: "{app}"
Name: "{group}\Désinstaller {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Orchestrateur"; Filename: "{app}\{#MyAppExeHUD}"; Parameters: "--workspace ""{code:GetWorkspaceRoot}"""; Tasks: desktopicon; WorkingDir: "{app}"

[Run]
Filename: "{app}\{#MyAppExeHUD}"; Description: "Lancer {#MyAppName} (HUD)"; Flags: postinstall nowait skipifsilent; Parameters: "--workspace ""{code:GetWorkspaceRoot}"""

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\Orchestrateur"

[Code]
function GetWorkspaceRoot(Param: String): String;
begin
  Result := ExpandConstant('{userappdata}\Orchestrateur\workspace');
end;

procedure InitializeUserWorkspace;
var
  Root, ConfigDir, ConfigFile, ExampleFile: String;
begin
  Root := ExpandConstant('{userappdata}\Orchestrateur\workspace');
  ConfigDir := Root + '\config';
  ConfigFile := ConfigDir + '\orchestrator.toml';
  ExampleFile := ExpandConstant('{app}\workspace\orchestrator.toml.example');

  if not DirExists(Root) then
    ForceDirectories(Root);
  if not DirExists(Root + '\memories') then
    ForceDirectories(Root + '\memories');
  if not DirExists(Root + '\logs') then
    ForceDirectories(Root + '\logs');
  if not DirExists(ConfigDir) then
    ForceDirectories(ConfigDir);

  if (not FileExists(ConfigFile)) and FileExists(ExampleFile) then
    CopyFile(ExampleFile, ConfigFile, False);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    InitializeUserWorkspace;
end;