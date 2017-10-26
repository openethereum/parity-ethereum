// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

export default {
  busy: `你正在升級到Parity最新版本{newversion}。請等待升級過程完成。`,
  // Your upgrade to Parity {newversion} is currently in progress. Please wait until the process completes.
  button: {
    close: `關閉`, // close
    done: `完成`, // done
    upgrade: `現在升級`// upgrade now
  },
  completed: `你升級到Parity最新版本{newversion}的操作已經完成。點選“完成”將自動重新載入這個應用。`,
  // Your upgrade to Parity {newversion} has been successfully completed. Click "done" to automatically reload the application.
  consensus: {
    capable: `你當前的Parity版本能夠處理網路請求。`,
    // Your current Parity version is capable of handling the network requirements.
    capableUntil: `你當前的Parity版本能夠處理直到第{blockNumber}個區塊的網路請求。`,
    // Your current Parity version is capable of handling the network requirements until block {blockNumber}
    incapableSince: `你當前的Parity版本能夠處理第{blockNumber}個區塊以後的網路請求。`,
    // Your current Parity version is incapable of handling the network requirements since block {blockNumber}
    unknown: `你當前的Parity版本能夠處理網路請求。`
    // Your current Parity version is capable of handling the network requirements.
  },
  failed: `升級到Parity最新版本{newversion}遇到錯誤，升級失敗。`,
  // Your upgrade to Parity {newversion} has failed with an error.
  info: {
    currentVersion: `你現在正在執行{currentversion}版本。`, // You are currently running {currentversion}
    next: `點選“現在升級”，開始Parity升級。`, // Proceed with "upgrade now" to start your Parity upgrade.
    upgrade: `可以升級到最新版本{newversion}`, // An upgrade to version {newversion} is available
    welcome: `迎來到Parity升級指南，讓你享受無縫升級到Parity最新版本的體驗。`
    // Welcome to the Parity upgrade wizard, allowing you a completely seamless upgrade experience to the next version of Parity.歡
  },
  step: {
    completed: `升級完成`, // upgrade completed
    error: `錯誤`, // error
    info: `可以升級`, // upgrade available
    updating: `升級Parity`// upgrading parity
  },
  version: {
    unknown: `未知`// unknown
  }
};
