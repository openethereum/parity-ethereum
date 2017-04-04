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
  busy: `Your upgrade to Parity {newversion} is currently in progress. Please wait until the process completes.`,
  button: {
    close: `close`,
    done: `done`,
    upgrade: `upgrade now`
  },
  completed: `Your upgrade to Parity {newversion} has been successfully completed. Click "done" to automatically reload the application.`,
  consensus: {
    capable: `Your current Parity version is capable of handling the network requirements.`,
    capableUntil: `Your current Parity version is capable of handling the network requirements until block {blockNumber}`,
    incapableSince: `Your current Parity version is incapable of handling the network requirements since block {blockNumber}`,
    unknown: `Your current Parity version is capable of handling the network requirements.`
  },
  failed: `Your upgrade to Parity {newversion} has failed with an error.`,
  info: {
    currentVersion: `You are currently running {currentversion}`,
    next: `Proceed with "upgrade now" to start your Parity upgrade.`,
    upgrade: `An upgrade to version {newversion} is available`,
    welcome: `Welcome to the Parity upgrade wizard, allowing you a completely seamless upgrade experience to the next version of Parity.`
  },
  step: {
    completed: `upgrade completed`,
    error: `error`,
    info: `upgrade available`,
    updating: `upgrading parity`
  },
  version: {
    unknown: `unknown`
  }
};
