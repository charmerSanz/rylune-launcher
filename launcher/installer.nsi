Unicode true
Name "Rylune Client"
OutFile "release-files\RyluneClient-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Rylune Client"
RequestExecutionLevel user
Icon "icons\icon.ico"
Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles
Section "Rylune Client"
  SetOutPath "$INSTDIR"
  File "release-files\RyluneClient.exe"
  CreateDirectory "$SMPROGRAMS\Rylune Client"
  CreateShortCut "$SMPROGRAMS\Rylune Client\Rylune Client.lnk" "$INSTDIR\RyluneClient.exe"
  CreateShortCut "$DESKTOP\Rylune Client.lnk" "$INSTDIR\RyluneClient.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RyluneNative" "DisplayName" "Rylune Client"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RyluneNative" "UninstallString" '"$INSTDIR\Uninstall.exe"'
SectionEnd
Section "Uninstall"
  Delete "$INSTDIR\RyluneClient.exe"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$DESKTOP\Rylune Client.lnk"
  Delete "$SMPROGRAMS\Rylune Client\Rylune Client.lnk"
  RMDir "$SMPROGRAMS\Rylune Client"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RyluneNative"
  RMDir "$INSTDIR"
SectionEnd
