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
    overview_0: `你現在所看到的背景圖案在你的Parity安裝中是獨一無二的。每次創造一個新的Signer令牌都會改變一次圖案。這也保證了去中性化應用不能偽裝成可信的樣子。`,
    // The background pattern you can see right now is unique to your Parity installation. It will change every time you create a new
    // Signer token. This is so that decentralized applications cannot pretend to be trustworthy.
    overview_1: `選擇一個你喜歡的圖案並記住它的樣子。這個圖案從現在開始會經常出現，除非你清空了瀏覽器的快取或者使用了新的Signer令牌。`,
    // Pick a pattern you like and memorize it. This Pattern will always be shown from now on, unless you clear your browser cache or
    // use a new Signer token.
    label: `背景` // background
  },
  parity: {
    chains: {
      chain_classic: `將Parity同步至以太坊經典網路`, // Parity syncs to the Ethereum Classic network
      chain_dev: `將Parity使用一條本地開發用區塊鏈`, // Parity uses a local development chain
      chain_expanse: `將Parity同步至Expanse網路`, // Parity syncs to the Expanse network
      chain_musicoin: `將Parity同步至Musicoin網路`, // Parity syncs to the Musicoin network
      chain_foundation: `將Parity同步至以太坊基金會發起的以太坊網路`, // Parity syncs to the Ethereum network launched by the Ethereum Foundation
      chain_kovan: `將Parity同步至Kovan測試網路`, // Parity syncs to the Kovan test network
      chain_olympic: `將Parity同步至Olympic測試網路`, // Parity syncs to the Olympic test network
      chain_ropsten: `將Parity同步至Ropsten測試網路`, // Parity syncs to the Ropsten test network
      cmorden_kovan: `將Parity同步至Morden（經典）測試網路`, // Parity syncs to Morden (Classic) test network
      hint: `Parity節點同步的區塊鏈`, // the chain for the Parity node to sync to
      label: `將同步的區塊鏈/網路` // chain/network to sync
    },
    languages: {
      hint: `此介面顯示的語言`, // the language this interface is displayed with
      label: `介面語言` // UI language
    },
    loglevels: `選擇一個不同的logs層次`, // Choose the different logs level.
    modes: {
      hint: `Parity節點的同步模式`, // the syncing mode for the Parity node
      label: `執行模式`, // mode of operation
      mode_active: `Parity持續地同步區塊鏈`, // Parity continuously syncs the chain
      mode_dark: `Parity只有在RPC啟用時才同步`, // Parity syncs only when the RPC is active
      mode_offline: `Parity不同步`, // Parity doesn't sync
      mode_passive: `Parity初始同步，然後進入休眠並有規律地再同步` // Parity syncs initially, then sleeps and wakes regularly to resync
    },
    overview_0: `通過此介面控制Parity節點設定和同步設定`, // Control the Parity node settings and nature of syncing via this interface.
    label: `parity` // parity
  },
  proxy: {
    details_0: `除了通過IP地址和埠來訪問Parity，你也能通過.parity子域名來使用Parity，訪問 {homeProxy}。為了設定基於子域名的路由，你需要新增相關的代理記錄至你的瀏覽器。`,
    // Instead of accessing Parity via the IP address and port, you will be able to access it via the .parity subdomain, by visiting
    // {homeProxy}. To setup subdomain-based routing, you need to add the relevant proxy entries to your browser,
    details_1: `如果想了解如何配置代理，教程已提供在{windowsLink}，{macOSLink}和{ubuntuLink}。`,
    // To learn how to configure the proxy, instructions are provided for {windowsLink}, {macOSLink} or {ubuntuLink}.
    details_macos: `macOS`, // macOS
    details_ubuntu: `Ubuntu`, // Ubuntu
    details_windows: `Windows`, // Windows
    overview_0: `代理設定使你可以通過一個可記憶的地址來訪問Parity和所有相關的去中性化應用。`,
    // The proxy setup allows you to access Parity and all associated decentralized applications via memorable addresses.
    label: `代理` // proxy
  },
  views: {
    accounts: {
      description: `一個此Parity例項所關聯和匯入的所有帳戶的列表。傳送交易、接收流入價值、管理你的帳目和資助你的帳戶。`,
      // A list of all the accounts associated with and imported into this Parity instance. Send transactions, receive incoming values,
      // manage your balances and fund your accounts.
      label: `帳戶` // Accounts
    },
    addresses: {
      description: `一個此Parity例項管理的所有聯絡人和地址簿記錄的列表。只需點選一個按鈕就可以觀察帳戶並獲得所有交易相關的資訊。`,
      // A list of all contacts and address book entries managed by this Parity instance. Watch accounts and have the details available
      // at the click of a button when transacting.
      label: `地址簿` // Addressbook
    },
    apps: {
      description: `與整個底層網路交流的分散式應用。新增應用，管理你的應用庫和與網路上的其他應用進行互動。`,
      // Decentralized applications that interact with the underlying network. Add applications, manage you application portfolio and
      // interact with application from around the network.
      label: `應用` // Applications
    },
    contracts: {
      description: `觀察和互動已經被部署在網路上的特定合約。這是一個更注重技術的環境，特別為可以理解合約內部執行機制的高階使用者所設立。`,
      // Watch and interact with specific contracts that have been deployed on the network. This is a more technically-focused environment,
      // specifically for advanced users that understand the inner working of certain contracts.
      label: `合約` // Contracts
    },
    overview_0: `僅可視部分對你可用的應用來管理應用介面`,
    // Manage the available application views using only the parts of the application applicable to you.
    overview_1: `你是終端使用者？預設設定為初學者和高階使用者進行了相同的設定。`,
    // Are you an end-user? The defaults are setup for both beginner and advanced users alike.
    overview_2: `你是開發者？新增一些功能來管理合約和與應用部署互動。`,
    // Are you a developer? Add some features to manage contracts and interact with application deployments.
    overview_3: `你是礦工或者運營一個大型節點？新增一些功能來讓你獲得更多有關節點執行的資訊。`,
    // Are you a miner or run a large-scale node? Add the features to give you all the information needed to watch the node operation.
    settings: {
      description: `此介面。允許你自定義應用的選項、執行、視覺化和感官。`,
      // This view. Allows you to customize the application in term of options, operation and look and feel.
      label: `設定` // Settings
    },
    signer: {
      description: `這個應用安全交易管理區域，你可以通過任何從本應用和其他分散式應用發起的即將傳送的交易`,
      // The secure transaction management area of the application where you can approve any outgoing transactions made
      // from the application as well as those placed into the queue by decentralized applications.
      label: `Signer` // Signer
    },
    status: {
      description: `觀察Parity節點現在的執行情況：網路連線數、實際執行例項的Logs和具體挖礦資訊（如果已開啟並設定）`,
      // See how the Parity node is performing in terms of connections to the network, logs from the actual running instance
      // and details of mining (if enabled and configured).
      label: `狀態` // Status
    },
    label: `視窗`, // views
    home: {
      label: `首頁` // Home
    }
  },
  label: `設定` // settings
};
