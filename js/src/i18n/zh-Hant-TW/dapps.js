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
  add: {
    builtin: {
      desc: `Parity團隊開發的實驗性的，用以展示dapp的效能、整合、實驗性特性和控制特定網路的的客戶端行為`,
      // Experimental applications developed by the Parity team to show off dapp capabilities, integration, experimental features and to control certain network-wide client behaviour.
      label: `與Parity繫結的應用`// Applications bundled with Parity
    },
    label: `visible applications可見的應用`, // visible applications
    local: {
      desc: `All applications installed locally on the machine by the user for access by the Parity client.`,
      label: `本地可用的應用`// Applications locally available
    },
    network: {
      desc: `這些應用與Parity沒有關聯，也不是Parity釋出的。 它們是由各自的作者控制的。 在使用以前，請確保你理解每個應用的目標。`,
      // These applications are not affiliated with Parity nor are they published by Parity.Each remain under the control of their respective authors.Please ensure that you understand the goals for each application before interacting.
      label: `全球網路上的應用`// Applications on the global network
    }
  },
  button: {
    edit: `編輯`, // edit
    permissions: `許可`// permissions
  },
  external: {
    accept: `我理解這些應用和Parity沒有關聯`, // I understand that these applications are not affiliated with Parity
    warning: `第三方開發者開發的應用與Parity沒有關聯，也不是Parity釋出的。 它們是由各自的作者控制的。 在使用以前，請確保你理解每個應用的目標。`
    // Applications made available on the network by 3rd-party authors are not affiliated with Parity nor are they published by Parity. Each remain under the control of their respective authors. Please ensure that you understand the goals for each before interacting.
  },
  label: `去中心化應用`, // Decentralized Applications
  permissions: {
    label: `可見的dapp帳戶`// visible dapp accounts
  }
};
