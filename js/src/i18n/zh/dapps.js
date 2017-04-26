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
      desc: `Parity团队开发的实验性的，用以展示dapp的性能、集成、实验性特性和控制特定网络的的客户端行为`,
      // Experimental applications developed by the Parity team to show off dapp capabilities, integration, experimental features and to control certain network-wide client behaviour.
      label: `与Parity绑定的应用`// Applications bundled with Parity
    },
    label: `visible applications可见的应用`, // visible applications
    local: {
      desc: `All applications installed locally on the machine by the user for access by the Parity client.`,
      label: `本地可用的应用`// Applications locally available
    },
    network: {
      desc: `这些应用与Parity没有关联，也不是Parity发布的。 它们是由各自的作者控制的。 在使用以前，请确保你理解每个应用的目标。`,
      // These applications are not affiliated with Parity nor are they published by Parity.Each remain under the control of their respective authors.Please ensure that you understand the goals for each application before interacting.
      label: `全球网络上的应用`// Applications on the global network
    }
  },
  button: {
    edit: `编辑`, // edit
    permissions: `许可`// permissions
  },
  external: {
    accept: `我理解这些应用和Parity没有关联`, // I understand that these applications are not affiliated with Parity
    warning: `第三方开发者开发的应用与Parity没有关联，也不是Parity发布的。 它们是由各自的作者控制的。 在使用以前，请确保你理解每个应用的目标。`
    // Applications made available on the network by 3rd-party authors are not affiliated with Parity nor are they published by Parity. Each remain under the control of their respective authors. Please ensure that you understand the goals for each before interacting.
  },
  label: `去中心化应用`, // Decentralized Applications
  permissions: {
    label: `可见的dapp账户`// visible dapp accounts
  }
};
