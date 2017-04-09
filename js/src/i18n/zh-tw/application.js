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
    error: `錯誤：這個應用不能也不應該載入到內建框架中`//ERROR: This application cannot and should not be loaded in an embedded iFrame
  },
  status: {
    consensus: {
      capable: `Capable`,
      capableUntil: `Capable until #{blockNumber}`,
      incapableSince: `Incapable since #{blockNumber}`,
      unknown: `Unknown capability未知的`
    },
    upgrade: `Upgrade升級`
  }
};
