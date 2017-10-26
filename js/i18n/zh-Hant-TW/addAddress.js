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
  button: {
    add: `儲存地址`, // Save Address
    close: `取消` // Cancel
  },
  header: `如果想在地址簿中新增一條新的記錄，你需要擁有帳戶的網路地址並提供一個的描述（可選）。一旦新增，記錄就可以體現在你的地址簿中。`,
  // To add a new entry to your addressbook, you need the network
  // address of the account and can supply an optional description.
  // Once added it will reflect in your address book.
  input: {
    address: {
      hint: `記錄的網路地址`, // the network address for the entry
      label: `網路地址` // network address
    },
    description: {
      hint: `記錄的詳細描述`, // an expanded description for the entry
      label: `（可選）地址描述` // (optional) address description
    },
    name: {
      hint: `記錄的名字`, // a descriptive name for the entry
      label: `地址名` // address name
    }
  },
  label: `新增已儲存的地址` // add saved address
};
