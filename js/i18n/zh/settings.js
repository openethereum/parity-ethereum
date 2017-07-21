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
  background: {
    button_more: `生成更多`, // generate more
    overview_0: `你现在所看到的背景图案在你的Parity安装中是独一无二的。每次创造一个新的Signer令牌都会改变一次图案。这也保证了去中性化应用不能伪装成可信的样子。`,
    // The background pattern you can see right now is unique to your Parity installation. It will change every time you create a new
    // Signer token. This is so that decentralized applications cannot pretend to be trustworthy.
    overview_1: `选择一个你喜欢的图案并记住它的样子。这个图案从现在开始会经常出现，除非你清空了浏览器的缓存或者使用了新的Signer令牌。`,
    // Pick a pattern you like and memorize it. This Pattern will always be shown from now on, unless you clear your browser cache or
    // use a new Signer token.
    label: `背景` // background
  },
  parity: {
    chains: {
      chain_classic: `将Parity同步至以太坊经典网络`, // Parity syncs to the Ethereum Classic network
      chain_dev: `将Parity使用一条本地开发用区块链`, // Parity uses a local development chain
      chain_expanse: `将Parity同步至Expanse网络`, // Parity syncs to the Expanse network
      chain_foundation: `将Parity同步至以太坊基金会发起的以太坊网络`, // Parity syncs to the Ethereum network launched by the Ethereum Foundation
      chain_kovan: `将Parity同步至Kovan测试网络`, // Parity syncs to the Kovan test network
      chain_olympic: `将Parity同步至Olympic测试网络`, // Parity syncs to the Olympic test network
      chain_ropsten: `将Parity同步至Ropsten测试网络`, // Parity syncs to the Ropsten test network
      cmorden_kovan: `将Parity同步至Morden（经典）测试网络`, // Parity syncs to Morden (Classic) test network
      hint: `Parity节点同步的区块链`, // the chain for the Parity node to sync to
      label: `将同步的区块链/网络` // chain/network to sync
    },
    languages: {
      hint: `此界面显示的语言`, // the language this interface is displayed with
      label: `界面语言` // UI language
    },
    loglevels: `选择一个不同的logs层次`, // Choose the different logs level.
    modes: {
      hint: `Parity节点的同步模式`, // the syncing mode for the Parity node
      label: `运行模式`, // mode of operation
      mode_active: `Parity持续地同步区块链`, // Parity continuously syncs the chain
      mode_dark: `Parity只有在RPC激活时才同步`, // Parity syncs only when the RPC is active
      mode_offline: `Parity不同步`, // Parity doesn't sync
      mode_passive: `Parity初始同步，然后进入休眠并有规律地再同步` // Parity syncs initially, then sleeps and wakes regularly to resync
    },
    overview_0: `通过此界面控制Parity节点设置和同步设置`, // Control the Parity node settings and nature of syncing via this interface.
    label: `parity` // parity
  },
  proxy: {
    details_0: `除了通过IP地址和端口来访问Parity，你也能通过.parity子域名来使用Parity，访问 {homeProxy}。为了设置基于子域名的路由，你需要添加相关的代理记录至你的浏览器。`,
    // Instead of accessing Parity via the IP address and port, you will be able to access it via the .parity subdomain, by visiting
    // {homeProxy}. To setup subdomain-based routing, you need to add the relevant proxy entries to your browser,
    details_1: `如果想了解如何配置代理，教程已提供在{windowsLink}，{macOSLink}和{ubuntuLink}。`,
    // To learn how to configure the proxy, instructions are provided for {windowsLink}, {macOSLink} or {ubuntuLink}.
    details_macos: `macOS`, // macOS
    details_ubuntu: `Ubuntu`, // Ubuntu
    details_windows: `Windows`, // Windows
    overview_0: `代理设置使你可以通过一个可记忆的地址来访问Parity和所有相关的去中性化应用。`,
    // The proxy setup allows you to access Parity and all associated decentralized applications via memorable addresses.
    label: `代理` // proxy
  },
  views: {
    accounts: {
      description: `一个此Parity实例所关联和导入的所有账户的列表。发送交易、接收流入价值、管理你的账目和资助你的账户。`,
      // A list of all the accounts associated with and imported into this Parity instance. Send transactions, receive incoming values,
      // manage your balances and fund your accounts.
      label: `账户` // Accounts
    },
    addresses: {
      description: `一个此Parity实例管理的所有联系人和地址簿记录的列表。只需点击一个按钮就可以观察账户并获得所有交易相关的信息。`,
      // A list of all contacts and address book entries managed by this Parity instance. Watch accounts and have the details available
      // at the click of a button when transacting.
      label: `地址簿` // Addressbook
    },
    apps: {
      description: `与整个底层网络交流的分布式应用。添加应用，管理你的应用库和与网络上的其他应用进行交互。`,
      // Decentralized applications that interact with the underlying network. Add applications, manage you application portfolio and
      // interact with application from around the network.
      label: `应用` // Applications
    },
    contracts: {
      description: `观察和交互已经被部署在网络上的特定合约。这是一个更注重技术的环境，特别为可以理解合约内部运行机制的高级用户所设立。`,
      // Watch and interact with specific contracts that have been deployed on the network. This is a more technically-focused environment,
      // specifically for advanced users that understand the inner working of certain contracts.
      label: `合约` // Contracts
    },
    overview_0: `仅可视部分对你可用的应用来管理应用界面`,
    // Manage the available application views using only the parts of the application applicable to you.
    overview_1: `你是终端用户？默认设置为初学者和高级用户进行了相同的设置。`,
    // Are you an end-user? The defaults are setup for both beginner and advanced users alike.
    overview_2: `你是开发者？添加一些功能来管理合约和与应用部署交互。`,
    // Are you a developer? Add some features to manage contracts and interact with application deployments.
    overview_3: `你是矿工或者运营一个大型节点？添加一些功能来让你获得更多有关节点运行的信息。`,
    // Are you a miner or run a large-scale node? Add the features to give you all the information needed to watch the node operation.
    settings: {
      description: `此界面。允许你自定义应用的选项、运行、可视化和感官。`,
      // This view. Allows you to customize the application in term of options, operation and look and feel.
      label: `设置` // Settings
    },
    signer: {
      description: `这个应用安全交易管理区域，你可以通过任何从本应用和其他分布式应用发起的即将发送的交易`,
      // The secure transaction management area of the application where you can approve any outgoing transactions made
      // from the application as well as those placed into the queue by decentralized applications.
      label: `Signer` // Signer
    },
    status: {
      description: `观察Parity节点现在的运行情况：网络连接数、实际运行实例的Logs和具体挖矿信息（如果已开启并设置）`,
      // See how the Parity node is performing in terms of connections to the network, logs from the actual running instance
      // and details of mining (if enabled and configured).
      label: `状态` // Status
    },
    label: `视窗`, // views
    home: {
      label: `首页` // Home
    }
  },
  label: `设置` // settings
};
