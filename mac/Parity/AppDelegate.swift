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

import Cocoa

@NSApplicationMain
@available(macOS, deprecated: 10.11)

class AppDelegate: NSObject, NSApplicationDelegate {
	@IBOutlet weak var statusMenu: NSMenu!
	@IBOutlet weak var startAtLogonMenuItem: NSMenuItem!

	let statusItem = NSStatusBar.system().statusItem(withLength: NSVariableStatusItemLength)
	var parityPid: Int32? = nil
	var commandLine: [String] = []
	let defaultDefaults = "{\"fat_db\":false,\"mode\":\"passive\",\"mode.alarm\":3600,\"mode.timeout\":300,\"pruning\":\"fast\",\"tracing\":false}"

	func menuAppPath() -> String {
		return Bundle.main.executablePath!
	}

	func parityPath() -> String {
		return Bundle.main.bundlePath + "/Contents/MacOS/parity"
	}

	func isAlreadyRunning() -> Bool {
		return NSRunningApplication.runningApplications(withBundleIdentifier: Bundle.main.bundleIdentifier!).count > 1

	}

	func isParityRunning() -> Bool {
		if let pid = self.parityPid {
			return kill(pid, 0) == 0
		}
		return false
	}

	func killParity() {
		if let pid = self.parityPid {
			kill(pid, SIGKILL)
		}
	}

	func openUI() {
		let parity = Process()
		parity.launchPath = self.parityPath()
		parity.arguments = self.commandLine
		parity.arguments!.append("ui")
		parity.launch()
	}

	func writeConfigFiles() {
		let basePath = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first?
			.appendingPathComponent(Bundle.main.bundleIdentifier!, isDirectory: true)

		if FileManager.default.fileExists(atPath: basePath!.path) {
			return
		}

		do {
			let defaultsFileDir = basePath?.appendingPathComponent("chains").appendingPathComponent("ethereum")
			let defaultsFile = defaultsFileDir?.appendingPathComponent("user_defaults")

			try FileManager.default.createDirectory(atPath: (defaultsFileDir?.path)!, withIntermediateDirectories: true, attributes: nil)
			if !FileManager.default.fileExists(atPath: defaultsFile!.path) {
				try defaultDefaults.write(to: defaultsFile!, atomically: false, encoding: String.Encoding.utf8)
			}

			let configFile = basePath?.appendingPathComponent("config.toml")
		}
		catch {}
	}

	func autostartEnabled() -> Bool {
		return itemReferencesInLoginItems().existingReference != nil
	}

	func itemReferencesInLoginItems() -> (existingReference: LSSharedFileListItem?, lastReference: LSSharedFileListItem?) {
		let itemUrl: UnsafeMutablePointer<Unmanaged<CFURL>?> = UnsafeMutablePointer<Unmanaged<CFURL>?>.allocate(capacity: 1)
		if let appUrl: NSURL = NSURL.fileURL(withPath: Bundle.main.bundlePath) as NSURL? {
			let loginItemsRef = LSSharedFileListCreate(
				nil,
				kLSSharedFileListSessionLoginItems.takeRetainedValue(),
				nil
				).takeRetainedValue() as LSSharedFileList?
			if loginItemsRef != nil {
				let loginItems: NSArray = LSSharedFileListCopySnapshot(loginItemsRef, nil).takeRetainedValue() as NSArray
				if(loginItems.count > 0)
				{
					let lastItemRef: LSSharedFileListItem = loginItems.lastObject as! LSSharedFileListItem
					for i in 0 ..< loginItems.count {
						let currentItemRef: LSSharedFileListItem = loginItems.object(at: i) as! LSSharedFileListItem
						if LSSharedFileListItemResolve(currentItemRef, 0, itemUrl, nil) == noErr {
							if let urlRef: NSURL =  itemUrl.pointee?.takeRetainedValue() {
								if urlRef.isEqual(appUrl) {
									return (currentItemRef, lastItemRef)
								}
							}
						}
					}
					//The application was not found in the startup list
					return (nil, lastItemRef)
				}
				else
				{
					let addAtStart: LSSharedFileListItem = kLSSharedFileListItemBeforeFirst.takeRetainedValue()
					return(nil, addAtStart)
				}
			}
		}
		return (nil, nil)
	}

	func toggleLaunchAtStartup() {
		let itemReferences = itemReferencesInLoginItems()
		let shouldBeToggled = (itemReferences.existingReference == nil)
		let loginItemsRef = LSSharedFileListCreate(
			nil,
			kLSSharedFileListSessionLoginItems.takeRetainedValue(),
			nil
			).takeRetainedValue() as LSSharedFileList?
		if loginItemsRef != nil {
			if shouldBeToggled {
				if let appUrl : CFURL = NSURL.fileURL(withPath: Bundle.main.bundlePath) as CFURL? {
					LSSharedFileListInsertItemURL(
						loginItemsRef,
						itemReferences.lastReference,
						nil,
						nil,
						appUrl,
						nil,
						nil
					)
				}
			} else {
				if let itemRef = itemReferences.existingReference {
					LSSharedFileListItemRemove(loginItemsRef,itemRef)
				}
			}
		}
	}

	func launchParity() {
		self.commandLine = CommandLine.arguments.dropFirst().filter({ $0 != "ui"})

		let processes = GetBSDProcessList()!
		let parityProcess = processes.index(where: {
			var name = $0.kp_proc.p_comm
			let str = withUnsafePointer(to: &name) {
				$0.withMemoryRebound(to: UInt8.self, capacity: MemoryLayout.size(ofValue: name)) {
				String(cString: $0)
				}
			}
			return str == "parity"
		})

		if parityProcess == nil {
			let parity = Process()
			let p = self.parityPath()
			parity.launchPath = p//self.parityPath()
			parity.arguments = self.commandLine
			parity.launch()
			self.parityPid = parity.processIdentifier
		} else {
			self.parityPid = processes[parityProcess!].kp_proc.p_pid
		}
	}

	func applicationDidFinishLaunching(_ aNotification: Notification) {
		if self.isAlreadyRunning() {
			openUI()
			NSApplication.shared().terminate(self)
			return
		}

		self.writeConfigFiles()
		self.launchParity()
		Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true, block: {_ in
			if !self.isParityRunning() {
				NSApplication.shared().terminate(self)
			}
		})

		let icon = NSImage(named: "statusIcon")
		icon?.isTemplate = true // best for dark mode
		statusItem.image = icon
		statusItem.menu = statusMenu
	}

	override func validateMenuItem(_ menuItem: NSMenuItem) -> Bool {
		if menuItem == self.startAtLogonMenuItem! {
			menuItem.state = self.autostartEnabled() ? NSOnState : NSOffState
		}
		return true
	}

	@IBAction func quitClicked(_ sender: NSMenuItem) {
		self.killParity()
		NSApplication.shared().terminate(self)
	}

	@IBAction func openClicked(_ sender: NSMenuItem) {
		self.openUI()
	}

	@IBAction func startAtLogonClicked(_ sender: NSMenuItem) {
		self.toggleLaunchAtStartup()
	}

}
