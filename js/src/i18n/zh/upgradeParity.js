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
  busy: `你正在升级到Parity最新版本{newversion}。请等待升级过程完成。`,
  // Your upgrade to Parity {newversion} is currently in progress. Please wait until the process completes.
  button: {
    close: `关闭`, // close
    done: `完成`, // done
    upgrade: `现在升级`// upgrade now
  },
  completed: `你升级到Parity最新版本{newversion}的操作已经完成。点击“完成”将自动重新加载这个应用。`,
  // Your upgrade to Parity {newversion} has been successfully completed. Click "done" to automatically reload the application.
  consensus: {
    capable: `你当前的Parity版本能够处理网络请求。`,
    // Your current Parity version is capable of handling the network requirements.
    capableUntil: `你当前的Parity版本能够处理直到第{blockNumber}个区块的网络请求。`,
    // Your current Parity version is capable of handling the network requirements until block {blockNumber}
    incapableSince: `你当前的Parity版本能够处理第{blockNumber}个区块以后的网络请求。`,
    // Your current Parity version is incapable of handling the network requirements since block {blockNumber}
    unknown: `你当前的Parity版本能够处理网络请求。`
    // Your current Parity version is capable of handling the network requirements.
  },
  failed: `升级到Parity最新版本{newversion}遇到错误，升级失败。`,
  // Your upgrade to Parity {newversion} has failed with an error.
  info: {
    currentVersion: `你现在正在运行{currentversion}版本。`, // You are currently running {currentversion}
    next: `点击“现在升级”，开始Parity升级。`, // Proceed with "upgrade now" to start your Parity upgrade.
    upgrade: `可以升级到最新版本{newversion}`, // An upgrade to version {newversion} is available
    welcome: `迎来到Parity升级指南，让你享受无缝升级到Parity最新版本的体验。`
    // Welcome to the Parity upgrade wizard, allowing you a completely seamless upgrade experience to the next version of Parity.欢
  },
  step: {
    completed: `升级完成`, // upgrade completed
    error: `错误`, // error
    info: `可以升级`, // upgrade available
    updating: `升级Parity`// upgrading parity
  },
  version: {
    unknown: `未知`// unknown
  }
};
