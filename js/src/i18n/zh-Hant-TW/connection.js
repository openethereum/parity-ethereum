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
  connectingAPI: `正在連線至Parity Secure API`, // Connecting to the Parity Secure API.
  connectingNode: `正在連線Parity節點。如果彈出任何資訊，請確認你的Parity節點正在執行並連線至網際網路。`,
  // Connecting to the Parity Node. If this informational message persists,
  // please ensure that your Parity node is running and reachable on the network.
  invalidToken: `無效的簽名令牌`, // invalid signer token
  noConnection: `無法連線至Parity Secure API。請升級的你的安全令牌或者生成一個新的，執行{newToken}並貼上生成的令牌到下方。`,
  // Unable to make a connection to the Parity Secure API. To update your secure
  // token or to generate a new one, run {newToken} and paste the generated token
  // into the space below.
  token: {
    hint: `一個Parity生成的令牌`, // a generated token from Parity
    label: `安全令牌` // secure token
  }
};
