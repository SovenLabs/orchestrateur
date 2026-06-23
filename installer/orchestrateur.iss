; Inno Setup 6 — Orchestrateur Windows installer
; Compilé via scripts/build-installer.ps1 (ISCC + defines StagingRoot / MyAppVersion)

#ifndef StagingRoot
  #define StagingRoot "..\dist\staging"
#endif

#ifndef MyAppVersion
  #define MyAppVersion "0.15.0"
#endif

#ifndef MyAppVersionFull
  #define MyAppVersionFull "0.15.0.0"
#endif

#define MyAppName "Orchestrateur"
#define MyAppPublisher "Sovën"
#define MyAppURL "https://github.com/SovenLabs/orchestrateur"
#define MyAppExeCLI "orch.exe"

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
UninstallDisplayIcon={app}\{#MyAppExeCLI}
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
Source: "{#StagingRoot}\orch.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\orchestrateur.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\orchestre.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\*.cmd"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\workspace\*"; DestDir: "{app}\workspace"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#StagingRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\communication.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\INSTALL.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingRoot}\app.ico"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}'))

[Icons]
Name: "{group}\Orchestrateur (daemon)"; Filename: "{app}\{#MyAppExeCLI}"; Parameters: "daemon run --workspace ""{code:GetWorkspaceRoot}"""; WorkingDir: "{app}"
Name: "{group}\Orchestrateur (CLI)"; Filename: "{app}\{#MyAppExeCLI}"; Parameters: "--help"; WorkingDir: "{app}"
Name: "{group}\Désinstaller {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Orchestrateur"; Filename: "{app}\{#MyAppExeCLI}"; Parameters: "daemon run --workspace ""{code:GetWorkspaceRoot}"""; Tasks: desktopicon; WorkingDir: "{app}"

[Run]
Filename: "{app}\{#MyAppExeCLI}"; Description: "Lancer le daemon {#MyAppName}"; Flags: postinstall nowait skipifsilent; Parameters: "daemon run --workspace ""{code:GetWorkspaceRoot}"""

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\Orchestrateur"

[Code]
function NeedsAddPath(Param: string): Boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
    OrigPath := '';
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

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