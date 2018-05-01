!include WinMessages.nsh

!define WND_CLASS "Parity"
!define WND_TITLE "Parity"
!define WAIT_MS 5000
!define SYNC_TERM 0x00100001

!define APPNAME "Parity"
!define COMPANYNAME "Parity Technologies"
!define DESCRIPTION "Fast, light, robust Ethereum implementation"
!define VERSIONMAJOR 1
!define VERSIONMINOR 12
!define VERSIONBUILD 0
!define ARGS ""
!define FIRST_START_ARGS "--mode=passive ui"

!addplugindir .\

!define HELPURL "https://paritytech.github.io/wiki/" # "Support Information" link
!define UPDATEURL "https://github.com/paritytech/parity/releases" # "Product Updates" link
!define ABOUTURL "https://github.com/paritytech/parity" # "Publisher" link
!define INSTALLSIZE 26120

!define termMsg "Installer cannot stop running ${WND_TITLE}.$\nDo you want to terminate process?"
!define stopMsg "Stopping ${WND_TITLE} Application"


RequestExecutionLevel admin ;Require admin rights on NT6+ (When UAC is turned on)

InstallDir "$PROGRAMFILES64\${COMPANYNAME}\${APPNAME}"

LicenseData "..\LICENSE"
Name "${COMPANYNAME} ${APPNAME}"
Icon "logo.ico"
outFile "installer.exe"

!include LogicLib.nsh

page license
page directory
page instfiles

!macro VerifyUserIsAdmin
UserInfo::GetAccountType
pop $0
${If} $0 != "admin" ;Require admin rights on NT4+
        messageBox mb_iconstop "Administrator rights required!"
        setErrorLevel 740 ;ERROR_ELEVATION_REQUIRED
        quit
${EndIf}
!macroend

!macro TerminateApp
    Push $0 ; window handle
    Push $1
    Push $2 ; process handle
    DetailPrint "$(stopMsg)"
    FindWindow $0 '${WND_CLASS}' ''
    IntCmp $0 0 done
    System::Call 'user32.dll::GetWindowThreadProcessId(i r0, *i .r1) i .r2'
    System::Call 'kernel32.dll::OpenProcess(i ${SYNC_TERM}, i 0, i r1) i .r2'
    SendMessage $0 ${WM_CLOSE} 0 0 /TIMEOUT=${TO_MS}
    System::Call 'kernel32.dll::WaitForSingleObject(i r2, i ${WAIT_MS}) i .r1'
    IntCmp $1 0 close
    MessageBox MB_YESNOCANCEL|MB_ICONEXCLAMATION "$(termMsg)" /SD IDYES IDYES terminate IDNO close
    System::Call 'kernel32.dll::CloseHandle(i r2) i .r1'
    Quit
  terminate:
    System::Call 'kernel32.dll::TerminateProcess(i r2, i 0) i .r1'
  close:
    System::Call 'kernel32.dll::CloseHandle(i r2) i .r1'
  done:
    Pop $2
    Pop $1
    Pop $0
!macroend

function .onInit
	setShellVarContext all
	!insertmacro VerifyUserIsAdmin
functionEnd

section "install"
	# Files for the install directory - to build the installer, these should be in the same directory as the install script (this file)
	setOutPath $INSTDIR

	# Close parity if running
	!insertmacro TerminateApp

	# Files added here should be removed by the uninstaller (see section "uninstall")
	file /oname=parity.exe ..\target\x86_64-pc-windows-msvc\release\parity.exe
  file /oname=parity-evm.exe ..\target\x86_64-pc-windows-msvc\release\parity-evm.exe
  file /oname=ethstore.exe ..\target\x86_64-pc-windows-msvc\release\ethstore.exe
  file /oname=ethkey.exe ..\target\x86_64-pc-windows-msvc\release\ethkey.exe
	file /oname=ptray.exe ..\windows\ptray\x64\Release\ptray.exe

	file "logo.ico"
	# Add any other files for the install directory (license files, app data, etc) here

	# Uninstaller - See function un.onInit and section "uninstall" for configuration
	writeUninstaller "$INSTDIR\uninstall.exe"

	# Start Menu
	createDirectory "$SMPROGRAMS\${COMPANYNAME}"
	delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk"
	createShortCut "$SMPROGRAMS\${COMPANYNAME}\${APPNAME} Ethereum.lnk" "$INSTDIR\ptray.exe" "ui" "$INSTDIR\logo.ico"
	createShortCut "$DESKTOP\${APPNAME} Ethereum.lnk" "$INSTDIR\ptray.exe" "ui" "$INSTDIR\logo.ico"

	# Firewall remove rules if exists
	SimpleFC::AdvRemoveRule "Parity incoming peers (TCP:30303)"
	SimpleFC::AdvRemoveRule "Parity outgoing peers (TCP:30303)"
	SimpleFC::AdvRemoveRule       "Parity web queries (TCP:80)"
	SimpleFC::AdvRemoveRule  "Parity UDP discovery (UDP:30303)"

	# Firewall exception rules
	SimpleFC::AdvAddRule "Parity incoming peers (TCP:30303)" ""  6 1 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity" 30303    "" "" ""
	SimpleFC::AdvAddRule "Parity outgoing peers (TCP:30303)" ""  6 2 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity"    "" 30303 "" ""
	SimpleFC::AdvAddRule  "Parity UDP discovery (UDP:30303)" "" 17 2 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity"    "" 30303 "" ""

	# Registry information for add/remove programs
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayName" "${APPNAME} - ${DESCRIPTION}"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "InstallLocation" "$\"$INSTDIR$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayIcon" "$\"$INSTDIR\logo.ico$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "Publisher" "${COMPANYNAME}"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "HelpLink" "$\"${HELPURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLUpdateInfo" "$\"${UPDATEURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLInfoAbout" "$\"${ABOUTURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMajor" ${VERSIONMAJOR}
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMinor" ${VERSIONMINOR}
	# There is no option for modifying or repairing the install
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoModify" 1
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoRepair" 1
	# Set the INSTALLSIZE constant (!defined at the top of this script) so Add/Remove Programs can accurately report the size
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "EstimatedSize" ${INSTALLSIZE}

	WriteRegStr HKEY_CURRENT_USER "Software\Microsoft\Windows\CurrentVersion\Run" ${APPNAME} "$INSTDIR\ptray.exe ${ARGS}"
	DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "${APPNAME}"
	ExecShell "" "$INSTDIR\ptray.exe" "${FIRST_START_ARGS}"
sectionEnd

# Uninstaller

function un.onInit
	SetShellVarContext all

	#Verify the uninstaller - last chance to back out
	MessageBox MB_OKCANCEL "Permanently remove ${APPNAME}?" IDOK next
		Abort

	next:
	!insertmacro VerifyUserIsAdmin
functionEnd

section "uninstall"
	!insertmacro TerminateApp
	# Remove Start Menu launcher
	delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk"
	delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME} Ethereum.lnk"
	delete "$DESKTOP\${APPNAME} Ethereum.lnk"

	# Try to remove the Start Menu folder - this will only happen if it is empty
	rmDir "$SMPROGRAMS\${COMPANYNAME}"

	# Remove files
	delete $INSTDIR\parity.exe
  delete $INSTDIR\parity-evm.exe
  delete $INSTDIR\ethstore.exe
  delete $INSTDIR\ethkey.exe
	delete $INSTDIR\ptray.exe
	delete $INSTDIR\logo.ico

	# Always delete uninstaller as the last action
	delete $INSTDIR\uninstall.exe

	# Try to remove the install directory - this will only happen if it is empty
	rmDir $INSTDIR

	# Firewall exception rules
	SimpleFC::AdvRemoveRule "Parity incoming peers (TCP:30303)"
	SimpleFC::AdvRemoveRule "Parity outgoing peers (TCP:30303)"
	SimpleFC::AdvRemoveRule       "Parity web queries (TCP:80)"
	SimpleFC::AdvRemoveRule  "Parity UDP discovery (UDP:30303)"

	# Remove uninstaller information from the registry
	DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}"
	DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "${APPNAME}"
	DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${APPNAME}"
sectionEnd
