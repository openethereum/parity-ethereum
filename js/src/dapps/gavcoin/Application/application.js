// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const muiTheme = getMuiTheme(lightBaseTheme);

import { api } from '../parity';

import * as abis from '../../../contracts/abi';

import Accounts from '../Accounts';
import Actions, { ActionBuyIn, ActionRefund, ActionTransfer } from '../Actions';
import Events from '../Events';
import Loading from '../Loading';
import Status from '../Status';

const DIVISOR = 10 ** 6;

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    contract: PropTypes.object,
    instance: PropTypes.object,
    muiTheme: PropTypes.object
  };

  state = {
    action: null,
    address: null,
    accounts: [],
    blockNumber: new BigNumber(-1),
    ethBalance: new BigNumber(0),
    gavBalance: new BigNumber(0),
    instance: null,
    loading: true,
    price: null,
    remaining: null,
    totalSupply: null
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    const { accounts, address, blockNumber, gavBalance, loading, price, remaining, totalSupply } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    return (
      <div>
        { this.renderModals() }
        <Status
          address={ address }
          blockNumber={ blockNumber }
          gavBalance={ gavBalance }
          price={ price }
          remaining={ remaining }
          totalSupply={ totalSupply }>
          <Accounts
            accounts={ accounts } />
        </Status>
        <Actions
          gavBalance={ gavBalance }
          onAction={ this.onAction } />
        <Events
          accounts={ accounts } />
      </div>
    );
  }

  renderModals () {
    const { action, accounts, price } = this.state;

    switch (action) {
      case 'BuyIn':
        return (
          <ActionBuyIn
            accounts={ accounts }
            price={ price }
            onClose={ this.onActionClose } />
        );
      case 'Refund':
        return (
          <ActionRefund
            accounts={ accounts }
            price={ price }
            onClose={ this.onActionClose } />
        );
      case 'Transfer':
        return (
          <ActionTransfer
            accounts={ accounts }
            onClose={ this.onActionClose } />
        );
      default:
        return null;
    }
  }

  getChildContext () {
    const { contract, instance } = this.state;

    return {
      api,
      contract,
      instance,
      muiTheme
    };
  }

  onAction = (action) => {
    this.setState({
      action
    });
  }

  onActionClose = () => {
    this.setState({
      action: null
    });
  }

  onNewBlockNumber = (_error, blockNumber) => {
    const { instance, accounts } = this.state;

    if (_error) {
      console.error('onNewBlockNumber', _error);
      return;
    }

    Promise
      .all([
        instance.totalSupply.call(),
        instance.remaining.call(),
        instance.price.call()
      ])
      .then(([totalSupply, remaining, price]) => {
        this.setState({
          blockNumber,
          totalSupply,
          remaining,
          price
        });

        const gavQueries = accounts.map((account) => instance.balanceOf.call({}, [account.address]));
        const ethQueries = accounts.map((account) => api.eth.getBalance(account.address));

        return Promise.all([
          Promise.all(gavQueries),
          Promise.all(ethQueries)
        ]);
      })
      .then(([gavBalances, ethBalances]) => {
        this.setState({
          ethBalance: ethBalances.reduce((total, balance) => total.add(balance), new BigNumber(0)),
          gavBalance: gavBalances.reduce((total, balance) => total.add(balance), new BigNumber(0)),
          accounts: accounts.map((account, idx) => {
            const ethBalance = ethBalances[idx];
            const gavBalance = gavBalances[idx];

            account.ethBalance = api.util.fromWei(ethBalance).toFormat(3);
            account.gavBalance = gavBalance.div(DIVISOR).toFormat(6);
            account.hasGav = gavBalance.gt(0);

            return account;
          })
        });
      })
      .catch((error) => {
        console.error('onNewBlockNumber', error);
      });
  }

  attachInterface = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        console.log(`the registry was found at ${registryAddress}`);

        const registry = api.newContract(abis.registry, registryAddress).instance;

        return Promise
          .all([
            registry.getAddress.call({}, [api.util.sha3('gavcoin'), 'A']),
            api.eth.accounts(),
            null // api.personal.accountsInfo()
          ]);
      })
      .then(([address, addresses, infos]) => {
        infos = infos || {};
        console.log(`gavcoin was found at ${address}`);

        const contract = api.newContract(abis.gavcoin, address);

        this.setState({
          loading: false,
          address,
          contract,
          instance: contract.instance,
          accounts: addresses.map((address) => {
            const info = infos[address] || {};

            return {
              address,
              name: info.name,
              uuid: info.uuid
            };
          })
        });

        api.subscribe('eth_blockNumber', this.onNewBlockNumber);
      })
      .catch((error) => {
        console.error('attachInterface', error);
      });
  }
}
