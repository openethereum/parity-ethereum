
!define APPNAME "Parity"
!define COMPANYNAME "Ethcore"
!define DESCRIPTION "Fast, light, robust Ethereum implementation"
!define VERSIONMAJOR 1
!define VERSIONMINOR 4
!define VERSIONBUILD 0

!addplugindir .\

!define HELPURL "https://github.com/ethcore/parity/wiki" # "Support Information" link
!define UPDATEURL "https://github.com/ethcore/parity/releases" # "Product Updates" link
!define ABOUTURL "https://github.com/ethcore/parity" # "Publisher" link
!define INSTALLSIZE 26120

RequestExecutionLevel admin ;Require admin rights on NT6+ (When UAC is turned on)

InstallDir "$PROGRAMFILES64\${COMPANYNAME}\${APPNAME}"

LicenseData "..\LICENSE"
Name "${COMPANYNAME} ${APPNAME}"
Icon "logo.ico"
outFile "installer.exe"

!include LogicLib.nsh

page license
page directory
Page instfiles

!macro VerifyUserIsAdmin
UserInfo::GetAccountType
pop $0
${If} $0 != "admin" ;Require admin rights on NT4+
        messageBox mb_iconstop "Administrator rights required!"
        setErrorLevel 740 ;ERROR_ELEVATION_REQUIRED
        quit
${EndIf}
!macroend

function .onInit
	setShellVarContext all
	!insertmacro VerifyUserIsAdmin
functionEnd

section "install"
	# Files for the install directory - to build the installer, these should be in the same directory as the install script (this file)
	setOutPath $INSTDIR
	# Files added here should be removed by the uninstaller (see section "uninstall")
	file /oname=parity.exe ..\target\release\parity.exe
	file "logo.ico"
	file vc_redist.x64.exe

	ExecWait '"$INSTDIR\vc_redist.x64.exe"  /passive /norestart'
	# Add any other files for the install directory (license files, app data, etc) here

	# Uninstaller - See function un.onInit and section "uninstall" for configuration
	writeUninstaller "$INSTDIR\uninstall.exe"

	# Start Menu
	createDirectory "$SMPROGRAMS\${COMPANYNAME}"
	createShortCut "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk" "$INSTDIR\parity.exe" "ui" "$INSTDIR\logo.ico"

	# Firewall remove rules if exists
	SimpleFC::AdvRemoveRule "Parity incoming peers (TCP:30303)"
	SimpleFC::AdvRemoveRule "Parity outgoing peers (TCP:30303)"
	SimpleFC::AdvRemoveRule       "Parity web queries (TCP:80)"
	SimpleFC::AdvRemoveRule  "Parity UDP discovery (UDP:30303)"

	# Firewall exception rules
	SimpleFC::AdvAddRule "Parity incoming peers (TCP:30303)" ""  6 1 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity" 30303    "" "" ""
	SimpleFC::AdvAddRule "Parity outgoing peers (TCP:30303)" ""  6 2 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity"    "" 30303 "" ""
	SimpleFC::AdvAddRule       "Parity web queries (TCP:80)" ""  6 2 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity"    ""    80 "" ""
	SimpleFC::AdvAddRule  "Parity UDP discovery (UDP:30303)" "" 17 2 1 2147483647 1 "$INSTDIR\parity.exe" "" "" "Parity"    "" 30303 "" ""

	# Registry information for add/remove programs
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayName" "${COMPANYNAME} - ${APPNAME} - ${DESCRIPTION}"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "InstallLocation" "$\"$INSTDIR$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayIcon" "$\"$INSTDIR\logo.ico$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "Publisher" "$\"${COMPANYNAME}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "HelpLink" "$\"${HELPURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLUpdateInfo" "$\"${UPDATEURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLInfoAbout" "$\"${ABOUTURL}$\""
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayVersion" "$\"${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}$\""
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMajor" ${VERSIONMAJOR}
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMinor" ${VERSIONMINOR}
	# There is no option for modifying or repairing the install
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoModify" 1
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoRepair" 1
	# Set the INSTALLSIZE constant (!defined at the top of this script) so Add/Remove Programs can accurately report the size
	WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "EstimatedSize" ${INSTALLSIZE}
sectionEnd

# Uninstaller

function un.onInit
	SetShellVarContext all

	#Verify the uninstaller - last chance to back out
	MessageBox MB_OKCANCEL "Permanantly remove ${APPNAME}?" IDOK next
		Abort

	next:
	!insertmacro VerifyUserIsAdmin
functionEnd

section "uninstall"

	# Remove Start Menu launcher
	delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk"
	# Try to remove the Start Menu folder - this will only happen if it is empty
	rmDir "$SMPROGRAMS\${COMPANYNAME}"

	# Remove files
	delete $INSTDIR\parity.exe
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

sectionEnd
