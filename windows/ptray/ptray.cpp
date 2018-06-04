// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#define WIN32_LEAN_AND_MEAN 
#include <stdlib.h>
#include <malloc.h>
#include <memory.h>
#include <tchar.h>
#include <Windows.h>
#include <Shellapi.h>
#include <Shlwapi.h>
#include <tlhelp32.h>

#include <SDKDDKVer.h>

#include "resource.h"

#pragma comment(lib, "shlwapi.lib")

#define MAX_LOADSTRING 100
#define IDM_EXIT 100
#define IDM_OPEN 101
#define IDM_AUTOSTART 102
#define	WM_USER_SHELLICON WM_USER + 1

HANDLE parityHandle = INVALID_HANDLE_VALUE;
DWORD parityProcId = 0;
NOTIFYICONDATA nidApp;
WCHAR szTitle[MAX_LOADSTRING];
WCHAR szWindowClass[MAX_LOADSTRING];
LPCWCHAR commandLineFiltered = L"";

LPCWSTR cParityExe = _T("parity.exe");

ATOM MyRegisterClass(HINSTANCE hInstance);
bool InitInstance(HINSTANCE, int, LPWSTR cmdLine);
LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM);
void KillParity();
void OpenUI();
bool ParityIsRunning();
bool AutostartEnabled();
void EnableAutostart(bool enable);

bool GetParityExePath(TCHAR* dest, size_t destSize)
{
	if (!dest || MAX_PATH > destSize)
		return false;
	GetModuleFileName(NULL, dest, (DWORD)destSize);
	if (!PathRemoveFileSpec(dest))
		return false;
	return PathAppend(dest, _T("parity.exe")) == TRUE;
}

bool GetTrayExePath(TCHAR* dest, size_t destSize)
{
	if (!dest || MAX_PATH > destSize)
		return false;
	GetModuleFileName(NULL, dest, (DWORD)destSize);
	return true;
}

int APIENTRY wWinMain(_In_ HINSTANCE hInstance,
	_In_opt_ HINSTANCE hPrevInstance,
	_In_ LPWSTR    lpCmdLine,
	_In_ int       nCmdShow)
{
	UNREFERENCED_PARAMETER(hPrevInstance);
	UNREFERENCED_PARAMETER(lpCmdLine);

	CreateMutex(0, FALSE, _T("Local\\ParityTray"));
	if (GetLastError() == ERROR_ALREADY_EXISTS) {
		// open the UI
		OpenUI();
		return 0;
	}

	LoadStringW(hInstance, IDS_APP_TITLE, szTitle, MAX_LOADSTRING);
	LoadStringW(hInstance, IDC_PTRAY, szWindowClass, MAX_LOADSTRING);
	MyRegisterClass(hInstance);

	if (!InitInstance(hInstance, nCmdShow, lpCmdLine))
		return FALSE;

	HACCEL hAccelTable = LoadAccelerators(hInstance, MAKEINTRESOURCE(IDC_PTRAY));
	MSG msg;
	// Main message loop:
	while (GetMessage(&msg, nullptr, 0, 0))
	{
		if (!TranslateAccelerator(msg.hwnd, hAccelTable, &msg))
		{
			TranslateMessage(&msg);
			DispatchMessage(&msg);
		}
	}

	return (int)msg.wParam;
}

ATOM MyRegisterClass(HINSTANCE hInstance)
{
	WNDCLASSEXW wcex;

	wcex.cbSize = sizeof(WNDCLASSEX);

	wcex.style = CS_HREDRAW | CS_VREDRAW;
	wcex.lpfnWndProc = WndProc;
	wcex.cbClsExtra = 0;
	wcex.cbWndExtra = 0;
	wcex.hInstance = hInstance;
	wcex.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE(IDI_PTRAY));
	wcex.hCursor = LoadCursor(nullptr, IDC_ARROW);
	wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
	wcex.lpszMenuName = MAKEINTRESOURCEW(IDC_PTRAY);
	wcex.lpszClassName = szWindowClass;
	wcex.hIconSm = LoadIcon(wcex.hInstance, MAKEINTRESOURCE(IDI_SMALL));

	return RegisterClassExW(&wcex);
}


bool InitInstance(HINSTANCE hInstance, int nCmdShow, LPWSTR cmdLine)
{
	if (lstrlen(cmdLine) > 0)
	{
		int commandLineArgs = 0;
		LPWSTR* commandLine = CommandLineToArgvW(cmdLine, &commandLineArgs);
		LPWSTR filteredArgs = new WCHAR[lstrlen(cmdLine) + 2];
		filteredArgs[0] = '\0';
		for (int i = 0; i < commandLineArgs; i++)
		{
			// Remove "ui" from command line
			if (lstrcmp(commandLine[i], L"ui") != 0)
			{
				lstrcat(filteredArgs, commandLine[i]);
				lstrcat(filteredArgs, L" ");
			}
		}
		commandLineFiltered = filteredArgs;
	}

	// Check if already running
	PROCESSENTRY32 entry;
	entry.dwSize = sizeof(PROCESSENTRY32);

	HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, NULL);
	if (Process32First(snapshot, &entry) == TRUE)
	{
		while (Process32Next(snapshot, &entry) == TRUE)
		{
			if (lstrcmp(entry.szExeFile, cParityExe) == 0)
			{
				parityHandle = OpenProcess(PROCESS_ALL_ACCESS, FALSE, entry.th32ProcessID);
				parityProcId = entry.th32ProcessID;
				break;
			}
		}
	}

	CloseHandle(snapshot);

	if (parityHandle == INVALID_HANDLE_VALUE)
	{
		// Launch parity
		TCHAR path[MAX_PATH] = { 0 };
		if (!GetParityExePath(path, MAX_PATH))
			return false;

		PROCESS_INFORMATION procInfo = { 0 };
		STARTUPINFO startupInfo = { sizeof(STARTUPINFO) };

		LPWSTR cmd = new WCHAR[lstrlen(cmdLine) + lstrlen(path) + 2];
		lstrcpy(cmd, path);
		lstrcat(cmd, _T(" "));
		lstrcat(cmd, cmdLine);
		if (!CreateProcess(nullptr, cmd, nullptr, nullptr, false, CREATE_NO_WINDOW, nullptr, nullptr, &startupInfo, &procInfo))
			return false;
		delete[] cmd;
		parityHandle = procInfo.hProcess;
		parityProcId = procInfo.dwProcessId;
	}

	HWND hWnd = CreateWindowW(szWindowClass, szTitle, 0,
		CW_USEDEFAULT, 0, CW_USEDEFAULT, 0, nullptr, nullptr, hInstance, nullptr);

	if (!hWnd)
		return false;

	HICON hMainIcon = LoadIcon(hInstance, (LPCTSTR)MAKEINTRESOURCE(IDI_PTRAY));

	nidApp.cbSize = sizeof(NOTIFYICONDATA); // sizeof the struct in bytes 
	nidApp.hWnd = (HWND)hWnd;              //handle of the window which will process this app. messages 
	nidApp.uID = IDI_PTRAY;           //ID of the icon that willl appear in the system tray 
	nidApp.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP; //ORing of all the flags 
	nidApp.hIcon = hMainIcon; // handle of the Icon to be displayed, obtained from LoadIcon 
	nidApp.uCallbackMessage = WM_USER_SHELLICON;
	LoadString(hInstance, IDS_CONTROL_PARITY, nidApp.szTip, MAX_LOADSTRING);
	Shell_NotifyIcon(NIM_ADD, &nidApp);

	SetTimer(hWnd, 0, 1000, nullptr);
	return true;

}

LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch (message)
	{
	case WM_USER_SHELLICON:
		// systray msg callback 
		POINT lpClickPoint;
		switch (LOWORD(lParam))
		{
		case WM_LBUTTONDOWN:
			OpenUI();
			break;
		case WM_RBUTTONDOWN:
			UINT uFlag = MF_BYPOSITION | MF_STRING;
			GetCursorPos(&lpClickPoint);
			HMENU hPopMenu = CreatePopupMenu();
			InsertMenu(hPopMenu, 0xFFFFFFFF, MF_BYPOSITION | MF_STRING, IDM_OPEN, _T("Open"));
			InsertMenu(hPopMenu, 0xFFFFFFFF, MF_SEPARATOR | MF_BYPOSITION, 0, nullptr);
			InsertMenu(hPopMenu, 0xFFFFFFFF, MF_BYPOSITION | MF_STRING, IDM_AUTOSTART, _T("Start at Login"));
			InsertMenu(hPopMenu, 0xFFFFFFFF, MF_SEPARATOR | MF_BYPOSITION, 0, nullptr);
			InsertMenu(hPopMenu, 0xFFFFFFFF, MF_BYPOSITION | MF_STRING, IDM_EXIT, _T("Exit"));
			bool autoStart = AutostartEnabled();
			CheckMenuItem(hPopMenu, IDM_AUTOSTART, autoStart ? MF_CHECKED : autoStart);

			SetForegroundWindow(hWnd);
			TrackPopupMenu(hPopMenu, TPM_LEFTALIGN | TPM_LEFTBUTTON | TPM_BOTTOMALIGN, lpClickPoint.x, lpClickPoint.y, 0, hWnd, NULL);
			return TRUE;

		}
		break;
	case WM_COMMAND:
	{
		int wmId = LOWORD(wParam);
		// Parse the menu selections:
		switch (wmId)
		{
		case IDM_EXIT:
			DestroyWindow(hWnd);
			break;
		case IDM_OPEN:
			OpenUI();
			break;
		case IDM_AUTOSTART:
			{
				bool autoStart = AutostartEnabled();
				EnableAutostart(!autoStart);
			}
			break;
		default:
			return DefWindowProc(hWnd, message, wParam, lParam);
		}
	}
	break;
	case WM_DESTROY:
		Shell_NotifyIcon(NIM_DELETE, &nidApp);
		KillParity();
		PostQuitMessage(0);
		break;
	case WM_TIMER:
		if (!ParityIsRunning())
			DestroyWindow(hWnd);
	default:
		return DefWindowProc(hWnd, message, wParam, lParam);
	}
	return 0;
}

void KillParity()
{
	DWORD procId = parityProcId;
	//This does not require the console window to be visible.
	if (AttachConsole(procId))
	{
		// Disable Ctrl-C handling for our program
		SetConsoleCtrlHandler(nullptr, true);
		GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0);
		FreeConsole();

		//Re-enable Ctrl-C handling or any subsequently started
		//programs will inherit the disabled state.
		SetConsoleCtrlHandler(nullptr, false);
	}
	WaitForSingleObject(parityHandle, INFINITE);
}

bool ParityIsRunning()
{
	return WaitForSingleObject(parityHandle, 0) == WAIT_TIMEOUT;
}

void OpenUI()
{
	// Launch parity
	TCHAR path[MAX_PATH] = { 0 };
	if (!GetParityExePath(path, MAX_PATH))
		return;

	PROCESS_INFORMATION procInfo = { 0 };
	STARTUPINFO startupInfo = { sizeof(STARTUPINFO) };

	LPWSTR args = new WCHAR[lstrlen(commandLineFiltered) + MAX_PATH + 2];
	lstrcpy(args, L"parity.exe ");
	lstrcat(args, commandLineFiltered);
	lstrcat(args, L" ui");
	CreateProcess(path, args, nullptr, nullptr, false, CREATE_NO_WINDOW, nullptr, nullptr, &startupInfo, &procInfo);
}

bool AutostartEnabled() {
	HKEY hKey;
	LONG lRes = RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_READ, &hKey);
	if (lRes != ERROR_SUCCESS)
		return false;

	WCHAR szBuffer[512];
	DWORD dwBufferSize = sizeof(szBuffer);
	ULONG nError;
	nError = RegQueryValueExW(hKey, L"Parity", 0, nullptr, (LPBYTE)szBuffer, &dwBufferSize);
	if (ERROR_SUCCESS != nError)
		return false;
	return true;
}

void EnableAutostart(bool enable) {
	HKEY hKey;
	LONG lRes = RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_WRITE, &hKey);
	if (lRes != ERROR_SUCCESS)
		return;

	if (enable) 
	{
		LPWSTR args = new WCHAR[lstrlen(commandLineFiltered) + MAX_PATH + 2];
		if (GetTrayExePath(args, MAX_PATH))
		{
			lstrcat(args, L" ");
			lstrcat(args, commandLineFiltered);
			RegSetValueEx(hKey, L"Parity", 0, REG_SZ, (LPBYTE)args, MAX_PATH);
		}
		delete[] args;
	}
	else
	{
		RegDeleteValue(hKey, L"Parity");
	}
}
