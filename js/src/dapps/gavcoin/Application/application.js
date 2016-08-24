import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const muiTheme = getMuiTheme(lightBaseTheme);

import registryAbi from '../abi/registry.json';
import gavcoinAbi from '../abi/gavcoin.json';

import Accounts from '../Accounts';
import Actions, { ActionBuyIn } from '../Actions';
import Events from '../Events';
import Loading from '../Loading';
import Status from '../Status';

const { Api } = window.parity;

const api = new Api(new Api.Transport.Http('/rpc/'));

const DIVISOR = 10 ** 6;

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    instance: PropTypes.object,
    muiTheme: PropTypes.object
  };

  state = {
    action: null,
    address: null,
    accounts: [],
    contract: null,
    instance: null,
    loading: true,
    blockNumber: null,
    totalSupply: null,
    remaining: null,
    price: null
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    return (
      <div>
        { this.renderLoading() }
        { this.renderInterface() }
      </div>
    );
  }

  renderLoading () {
    if (!this.state.loading) {
      return null;
    }

    return (
      <Loading />
    );
  }

  renderInterface () {
    if (this.state.loading) {
      return null;
    }

    return (
      <div>
        { this.renderModals() }
        <Status
          address={ this.state.address }
          blockNumber={ this.state.blockNumber }
          totalSupply={ this.state.totalSupply }
          remaining={ this.state.remaining }
          price={ this.state.price } />
        <Actions
          account={ this.state.accounts }
          onAction={ this.onAction } />
        <Accounts
          accounts={ this.state.accounts } />
        <Events />
      </div>
    );
  }

  renderModals () {
    switch (this.state.action) {
      case 'BuyIn':
        return (
          <ActionBuyIn
            accounts={ this.state.accounts }
            onClose={ this.onActionClose } />
        );
      default:
        return null;
    }
  }

  getChildContext () {
    return {
      api,
      instance: this.state.instance,
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

  onNewBlockNumber = (blockNumber) => {
    const { instance } = this.state;

    Promise
      .all([
        instance.totalSupply.call(),
        instance.remaining.call(),
        instance.price.call()
      ])
      .then(([totalSupply, remaining, price]) => {
        this.setState({
          blockNumber: blockNumber.toFormat(),
          totalSupply: totalSupply.toFormat(),
          remaining: remaining.toFormat(),
          price: price.div(DIVISOR).toFormat()
        });

        const { accounts } = this.state;
        const gavQueries = accounts.map((account) => instance.balanceOf.call({}, [account.address]));
        const ethQueries = accounts.map((account) => api.eth.getBalance(account.address));

        return Promise.all([
          Promise.all(gavQueries),
          Promise.all(ethQueries)
        ]);
      })
      .then(([gavBalances, ethBalances]) => {
        const { accounts } = this.state;

        this.setState({
          accounts: accounts.map((account, idx) => {
            const ethBalance = ethBalances[idx];
            const gavBalance = gavBalances[idx];

            account.ethBalance = Api.format.fromWei(ethBalance).toFormat(3);
            account.gavBalance = gavBalance.div(DIVISOR).toFormat(6);
            account.hasGav = gavBalance.gt(0);

            return account;
          })
        });
      });
  }

  attachInterface = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        console.log(`the registry was found at ${registryAddress}`);

        const registry = api.newContract(registryAbi, registryAddress).instance;

        return registry.getAddress.call({}, [Api.format.sha3('gavcoin'), 'A']);
      })
      .then((address) => {
        console.log(`gavcoin was found at ${address}`);

        const contract = api.newContract(gavcoinAbi, address);
        const instance = contract.instance;

        this.setState({
          address,
          contract,
          instance
        });

        return Promise.all([
          api.personal.listAccounts(),
          api.personal.accountsInfo()
        ]);
      })
      .then(([addresses, infos]) => {
        this.setState({
          loading: false,
          accounts: addresses.map((address) => {
            console.log(address, infos[address].name);
            return {
              address,
              name: infos[address].name || 'Unnamed',
              balance: 0
            };
          })
        });

        api.events.subscribe('eth.blockNumber', this.onNewBlockNumber);
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
