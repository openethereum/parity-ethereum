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
    button_more: `generate more`,
    overview_0: `The background pattern you can see right now is unique to your Parity installation. It will change every time you create a new Signer token. This is so that decentralized applications cannot pretend to be trustworthy.`,
    overview_1: `Pick a pattern you like and memorize it. This Pattern will always be shown from now on, unless you clear your browser cache or use a new Signer token.`,
    label: `background`
  },
  parity: {
    chains: {
      chain_classic: `Parity syncs to the Ethereum Classic network`,
      chain_dev: `Parity uses a local development chain`,
      chain_expanse: `Parity syncs to the Expanse network`,
      chain_musicoin: `Parity syncs to the Musicoin network`,
      chain_foundation: `Parity syncs to the Ethereum network launched by the Ethereum Foundation`,
      chain_kovan: `Parity syncs to the Kovan test network`,
      chain_olympic: `Parity syncs to the Olympic test network`,
      chain_ropsten: `Parity syncs to the Ropsten test network`,
      cmorden_kovan: `Parity syncs to Morden (Classic) test network`,
      hint: `the chain for the Parity node to sync to`,
      label: `chain/network to sync`
    },
    languages: {
      hint: `the language this interface is displayed with`,
      label: `language`
    },
    loglevels: `Choose the different logs level.`,
    modes: {
      hint: `the syncing mode for the Parity node`,
      label: `mode of operation`,
      mode_active: `Parity continuously syncs the chain`,
      mode_dark: `Parity syncs only when the RPC is active`,
      mode_offline: `Parity doesn't sync`,
      mode_passive: `Parity syncs initially, then sleeps and wakes regularly to resync`
    },
    overview_0: `Control the Parity node settings and nature of syncing via this interface.`,
    label: `parity`
  },
  proxy: {
    details_0: `Instead of accessing Parity via the IP address and port, you will be able to access it via the .web3.site subdomain, by visiting {homeProxy}. To setup subdomain-based routing, you need to add the relevant proxy entries to your browser,`,
    details_1: `To learn how to configure the proxy, instructions are provided for {windowsLink}, {macOSLink} or {ubuntuLink}.`,
    details_macos: `macOS`,
    details_ubuntu: `Ubuntu`,
    details_windows: `Windows`,
    overview_0: `The proxy setup allows you to access Parity and all associated decentralized applications via memorable addresses.`,
    label: `proxy`
  },
  views: {
    accounts: {
      description: `A list of all the accounts associated with and imported into this Parity instance. Send transactions, receive incoming values, manage your balances and fund your accounts.`,
      label: `Accounts`
    },
    addresses: {
      description: `A list of all contacts and address book entries managed by this Parity instance. Watch accounts and have the details available at the click of a button when transacting.`,
      label: `Addressbook`
    },
    apps: {
      description: `Decentralized applications that interact with the underlying network. Add applications, manage you application portfolio and interact with application from around the network.`,
      label: `Applications`
    },
    contracts: {
      description: `Watch and interact with specific contracts that have been deployed on the network. This is a more technically-focused environment, specifically for advanced users that understand the inner working of certain contracts.`,
      label: `Contracts`
    },
    overview_0: `Manage the available application views using only the parts of the application applicable to you.`,
    overview_1: `Are you an end-user? The defaults are setup for both beginner and advanced users alike.`,
    overview_2: `Are you a developer? Add some features to manage contracts and interact with application deployments.`,
    overview_3: `Are you a miner or run a large-scale node? Add the features to give you all the information needed to watch the node operation.`,
    settings: {
      description: `This view. Allows you to customize the application in term of options, operation and look and feel.`,
      label: `Settings`
    },
    signer: {
      description: `The secure transaction management area of the application where you can approve any outgoing transactions made from the application as well as those placed into the queue by decentralized applications.`,
      label: `Signer`
    },
    label: `views`,
    home: {
      label: `Home`
    },
    status: {
      label: `Status`
    }
  },
  label: `settings`
};
