[Setup]
AppName=oj-client
AppVersion=0.1.0
DefaultDirName={autopf}\oj-client
DisableDirPage=no
UsePreviousAppDir=no
CloseApplications=force
CloseApplicationsFilter=oj-client.exe
RestartApplications=no
DefaultGroupName=oj-client
OutputDir=installer-out
OutputBaseFilename=oj-client-setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\icon.ico

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Files]
; Build artifact (run `npm run build` then `npm run tauri build` before packaging)
Source: "src-tauri\target\release\oj-client.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "src-tauri\icons\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "src-tauri\resources\alarm.mp3"; DestDir: "{app}\resources"; Flags: ignoreversion

[Icons]
Name: "{group}\oj-client"; Filename: "{app}\oj-client.exe"; IconFilename: "{app}\icon.ico"
Name: "{autodesktop}\oj-client"; Filename: "{app}\oj-client.exe"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\oj-client.exe"; Description: "Launch oj-client"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\cache"
Type: filesandordirs; Name: "{app}\data"
Type: filesandordirs; Name: "{app}\resources"
Type: files; Name: "{app}\*.toml"

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  AppStatePath: string;
  RingPath: string;
  Content: string;
begin
  if CurStep <> ssPostInstall then
    exit;

  AppStatePath := ExpandConstant('{app}\appstate.toml');
  RingPath := ExpandConstant('{app}\resources\alarm.mp3');
  Content := 'ring_path = "' + RingPath + '"' + #13#10;

  SaveStringToFile(AppStatePath, Content, False);
end;