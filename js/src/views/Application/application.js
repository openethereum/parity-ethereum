import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { combineReducers, createStore } from 'redux';

import Api from '../../api';
import { eip20Abi, registryAbi, tokenRegAbi } from '../../services/abi';
import Errors, { errorReducer } from '../../ui/Errors';
import muiTheme from '../../ui/Theme';
import Tooltips, { tooltipReducer } from '../../ui/Tooltips';
import ParityBar from '../ParityBar';
import { FirstRun } from '../../modals';
import Status from './Status';
import TabBar from './TabBar';

import styles from './style.css';

const api = new Api(new Api.Transport.Http('/rpc/'));
const store = createStore(combineReducers({
  errors: errorReducer,
  tooltip: tooltipReducer
}), {});

const inFrame = window.parent !== window && window.parent.frames.length !== 0;

const ETH_TOKEN = {
  images: {
    small: '/images/contracts/ethereum-32.png',
    normal: '/images/contracts/ethereum-56.png'
  },
  name: 'Ethereum',
  tag: 'ΞTH'
};

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array,
    contacts: PropTypes.array,
    contracts: PropTypes.array,
    tokens: PropTypes.array,
    muiTheme: PropTypes.object,
    store: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    blockNumber: new BigNumber(-1),
    clientVersion: '',
    netChain: '',
    netPeers: {
      active: new BigNumber(0),
      connected: new BigNumber(0),
      max: new BigNumber(0)
    },
    showError: false,
    showFirst: false,
    accounts: [],
    contacts: [],
    contracts: [],
    errorMessage: null,
    tokens: []
  }

  componentWillMount () {
    this.retrieveAccounts();
    this.retrieveTokens();
    this.pollStatus();
  }

  render () {
    const { children } = this.props;
    const { blockNumber, clientVersion, netChain, netPeers } = this.state;
    const [root] = (window.location.hash || '').replace('#/', '').split('/');

    if (inFrame) {
      return (
        <div className={ styles.apperror }>
          ERROR: This application cannot and should not be loaded in an embedded iFrame
        </div>
      );
    } else if (root === 'app') {
      return (
        <div className={ styles.container }>
          { children }
          <ParityBar />
        </div>
      );
    }

    return (
      <div className={ styles.container }>
        { this.renderFirstRunDialog() }
        <Tooltips />
        <Errors />
        <TabBar />
        { children }
        <Status
          blockNumber={ blockNumber }
          clientVersion={ clientVersion }
          netChain={ netChain }
          netPeers={ netPeers } />
      </div>
    );
  }

  renderFirstRunDialog () {
    const { showFirst } = this.state;

    if (!showFirst) {
      return null;
    }

    return (
      <FirstRun
        onClose={ this.onCloseFirst } />
    );
  }

  getChildContext () {
    const { accounts, contacts, contracts, tokens } = this.state;

    return {
      api,
      accounts,
      contacts,
      contracts,
      tokens,
      muiTheme,
      store
    };
  }

  onCloseError = () => {
    this.setState({
      showError: false
    });
  }

  retrieveAccounts = () => {
    const nextTimeout = () => setTimeout(this.retrieveAccounts, 1000);

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo()
      ])
      .then(([addresses, infos]) => {
        const contacts = [];
        const accounts = [];

        Object.keys(infos).forEach((address) => {
          const { name, meta, uuid } = infos[address];

          if (uuid) {
            const account = this.state.accounts.find((_account) => _account.uuid === uuid) || {
              address,
              uuid,
              balances: [],
              txCount: 0
            };

            accounts.push(Object.assign(account, {
              meta,
              name
            }));
          } else {
            const contact = this.state.contacts.find((_contact) => _contact.address === address) || {
              address
            };

            contacts.push(Object.assign(contact, {
              meta,
              name
            }));
          }
        });

        this.setState({
          accounts,
          contacts
        }, nextTimeout);
      })
      .catch((error) => {
        console.error('retrieveAccounts', error);

        nextTimeout();
      });
  }

  retrieveBalances = () => {
    const { accounts, tokens } = this.state;

    return Promise
      .all(
        accounts.map((account) => {
          const { address } = account;

          return Promise.all([
            api.eth.getBalance(address),
            api.eth.getTransactionCount(address)
          ]);
        })
      )
      .then((balancesTxCounts) => {
        return Promise.all(
          balancesTxCounts.map(([balance, txCount], idx) => {
            const account = accounts[idx];

            account.txCount = txCount.sub(0x100000); // WHY?
            account.balances = [{
              token: ETH_TOKEN,
              value: balance.toString()
            }];

            return Promise.all(
              tokens.map((token) => {
                return token.contract.instance.balanceOf.call({}, [account.address]);
              })
            );
          })
        );
      })
      .then((balances) => {
        accounts.forEach((account, idx) => {
          const balanceOf = balances[idx];

          tokens.forEach((token, tidx) => {
            account.balances.push({
              token,
              value: balanceOf[tidx].toString()
            });
          });
        });

        this.setState({
          accounts,
          showFirst: accounts.length === 0
        });
      })
      .catch((error) => {
        console.error('retrieveBalances', error);
      });
  }

  retrieveTokens = () => {
    const contracts = {};
    const tokens = [];

    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        contracts.registry = api.newContract(registryAbi, registryAddress);

        return contracts.registry.instance.getAddress.call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        contracts.tokenreg = api.newContract(tokenRegAbi, tokenregAddress);

        return contracts.tokenreg.instance.tokenCount.call();
      })
      .then((tokenCount) => {
        const promises = [];

        while (promises.length < tokenCount.toNumber()) {
          promises.push(contracts.tokenreg.instance.token.call({}, [promises.length]));
        }

        return Promise.all(promises);
      })
      .then((_tokens) => {
        return Promise.all(
          _tokens.map((token) => {
            const contract = api.newContract(eip20Abi);
            contract.at(token[0]);

            tokens.push({
              address: token[0],
              format: token[2].toString(),
              images: {
                small: `/images/contracts/${token[3].toLowerCase()}-32.png`,
                normal: `/images/contracts/${token[3].toLowerCase()}-56.png`
              },
              supply: '0',
              tag: token[1],
              name: token[3],
              contract
            });

            return contract.instance.totalSupply.call();
          })
        );
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
        }, this.retrieveBalances);
      })
      .catch((error) => {
        console.error('retrieveTokens', error);
      });
  }

  pollStatus () {
    const nextTimeout = () => setTimeout(() => this.pollStatus(), 1000);

    Promise
      .all([
        api.eth.blockNumber(),
        api.web3.clientVersion(),
        api.ethcore.netChain(),
        api.ethcore.netPeers(),
        api.eth.syncing()
      ])
      .then(([blockNumber, clientVersion, netChain, netPeers, syncing]) => {
        if (blockNumber.gt(this.state.blockNumber)) {
          this.retrieveBalances();
        }

        this.setState({
          blockNumber,
          clientVersion,
          netChain,
          netPeers,
          syncing
        }, nextTimeout);
      })
      .catch((error) => {
        console.error('pollStatus', error);

        nextTimeout();
      });
  }

  onCloseFirst = () => {
    this.setState({
      showFirst: false
    });
  }
}
