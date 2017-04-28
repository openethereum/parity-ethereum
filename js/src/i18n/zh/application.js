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
  frame: {
    error: `错误：这个应用不能也不应该载入到内置框架中`// ERROR: This application cannot and should not be loaded in an embedded iFrame
  },
  status: {
    consensus: {
      capable: `可行`, // Capable
      capableUntil: `到第 #{blockNumber} 区块前可行`, // Capable until #{blockNumber}
      incapableSince: `自第 #{blockNumber} 区块后不可行`, // Incapable since #{blockNumber}
      unknown: `未知能力`// Unknown capability
    },
    upgrade: `升级`// Upgrade
  }
};
