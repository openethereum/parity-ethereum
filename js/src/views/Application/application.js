import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import darkBaseTheme from 'material-ui/styles/baseThemes/darkBaseTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

import Api from '../../api';
import { eip20Abi, registryAbi, tokenRegAbi } from '../../services/abi';
import { TooltipOverlay } from '../../ui/Tooltip';

import { FirstRun } from '../../modals';
import Status from './Status';
import TabBar from './TabBar';

import styles from './style.css';

const lightTheme = getMuiTheme(lightBaseTheme);
const muiTheme = getMuiTheme(darkBaseTheme);
const api = new Api(new Api.Transport.Http('/rpc/'));

muiTheme.stepper.textColor = '#eee';
muiTheme.stepper.disabledTextColor = '#777';
muiTheme.inkBar.backgroundColor = 'rgb(0, 151, 167)';
muiTheme.tabs = lightTheme.tabs;
muiTheme.tabs.backgroundColor = 'rgb(65, 65, 65)';
muiTheme.textField.disabledTextColor = muiTheme.textField.textColor;
muiTheme.toolbar = lightTheme.toolbar;
muiTheme.toolbar.backgroundColor = 'rgb(80, 80, 80)';

const ETH_TOKEN = {
  images: {
    small: 'images/tokens/ethereum-32x32.png'
  },
  name: 'Ethereum',
  tag: 'ΞTH'
};

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array,
    contracts: PropTypes.array,
    tokens: PropTypes.array,
    muiTheme: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    showFirst: false,
    accounts: [],
    contracts: [],
    tokens: []
  }

  componentWillMount () {
    this.retrieveBalances();
    this.retrieveTokens();
  }

  render () {
    return (
      <TooltipOverlay>
        <div className={ styles.container }>
          { this.renderFirstRunDialog() }
          <TabBar />
          { this.props.children }
          <Status />
        </div>
      </TooltipOverlay>
    );
  }

  renderFirstRunDialog () {
    if (!this.state.showFirst) {
      return null;
    }

    return (
      <FirstRun
        onClose={ this.onCloseFirst } />
    );
  }

  getChildContext () {
    return {
      api,
      accounts: this.state.accounts,
      contracts: this.state.contracts,
      tokens: this.state.tokens,
      muiTheme
    };
  }

  retrieveBalances = () => {
    const accounts = [];

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo()
      ])
      .then(([addresses, infos]) => {
        return Promise.all(addresses.map((address) => {
          const info = infos[address];

          accounts.push({
            address: address,
            balances: [],
            name: info.name,
            uuid: info.uuid,
            meta: info.meta
          });

          return api.eth.getBalance(address);
        }));
      })
      .then((balances) => {
        const promises = [];

        balances.forEach((balance, idx) => {
          accounts[idx].balances.push({
            token: ETH_TOKEN,
            value: balance.toString()
          });

          this.state.tokens.forEach((token) => {
            promises.push(token.contract.balanceOf.call({}, [accounts[idx].address]));
          });
        });

        return Promise.all(promises);
      })
      .then((balances) => {
        let idx = 0;

        accounts.forEach((account) => {
          this.state.tokens.forEach((token) => {
            const balance = balances[idx];

            account.balances.push({
              token,
              value: balance.toString()
            });

            idx++;
          });
        });

        this.setState({
          accounts,
          showFirst: accounts.length === 0
        });

        setTimeout(this.retrieveBalances, 2000);
      });
  }

  retrieveTokens = () => {
    const contracts = {};
    const tokens = [];

    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        contracts.registry = api.newContract(registryAbi).at(registryAddress);

        return contracts.registry
          .getAddress.call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        contracts.tokenreg = api.newContract(tokenRegAbi).at(tokenregAddress);

        return contracts.tokenreg.tokenCount.call();
      })
      .then((tokenCount) => {
        const promises = [];

        while (promises.length < tokenCount.toNumber()) {
          promises.push(contracts.tokenreg.token.call({}, [promises.length]));
        }

        return Promise.all(promises);
      })
      .then((_tokens) => {
        return Promise.all(_tokens.map((token) => {
          const contract = api.newContract(eip20Abi).at(token[0]);

          tokens.push({
            address: token[0],
            format: token[2].toString(),
            images: {
              small: `images/tokens/${token[3].toLowerCase()}-32x32.png`
            },
            supply: '0',
            tag: token[1],
            name: token[3],
            contract
          });

          return contract.totalSupply.call();
        }));
      })
      .then((supplies) => {
        supplies.forEach((supply, idx) => {
          tokens[idx].supply = supply.toString();
        });

        this.setState({
          tokens,
          contracts: Object.keys(contracts).map((name) => {
            const contract = contracts[name];

            return {
              name,
              contract,
              address: contract.address
            };
          }).concat(tokens)
        });
      });
  }

  onCloseFirst = () => {
    this.setState({
      showFirst: false
    });
  }
}
