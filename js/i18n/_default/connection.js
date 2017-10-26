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
  connectingAPI: `Connecting to the Parity Secure API.`,
  connectingNode: `Connecting to the Parity Node. If this informational message persists, please ensure that your Parity node is running and reachable on the network.`,
  invalidToken: `invalid signer token`,
  noConnection: `Unable to make a connection to the Parity Secure API. To update your secure token or to generate a new one, run {newToken} and paste the generated token into the space below.`,
  timestamp: `Ensure that both the Parity node and this machine connecting have computer clocks in-sync with each other and with a timestamp server, ensuring both successful token validation and block operations.`,
  token: {
    hint: `a generated token from Parity`,
    label: `secure token`
  }
};
