import React, { Component } from 'react';

import Api from '../../api';
import AccountSummary from './AccountSummary';
import Actions from './Actions';
import { CreateAccount } from '../../modals';
import { eip20Abi, registryAbi, tokenRegAbi } from '../../services/abi';
import Tooltip from '../../ui/Tooltip';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  state = {
    accounts: [],
    tokens: [],
    newDialog: false
  }

  componentDidMount () {
    // TODO: we should be getting data from a provider
    this._isMounted = true;
    this.retrieveAccounts();
  }

  componentWillUnmount () {
    this._isMounted = false;
  }

  render () {
    return (
      <div>
        <CreateAccount
          onClose={ this.onNewAccountClose }
          onUpdate={ this.onNewAccountUpdate }
          visible={ this.state.newDialog } />
        <Actions
          onNewAccount={ this.onNewAccountClick } />
        <div className={ styles.accounts }>
          { this.renderAccounts() }
        </div>
      </div>
    );
  }

  renderAccounts () {
    if (!this.state.accounts.length) {
      return null;
    }

    const firstTooltip = (
      <Tooltip
        top='80%'
        text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
    );

    return this.state.accounts.map((account, idx) => {
      return (
        <div
          className={ styles.account }
          key={ account.address }>
          <AccountSummary
            account={ account }
            tokens={ this.state.tokens }>
            { idx === 0 ? firstTooltip : null }
          </AccountSummary>
        </div>
      );
    });
  }

  onNewAccountClick = () => {
    this.setState({ newDialog: !this.state.newDialog });
  }

  onNewAccountClose = () => {
    this.onNewAccountClick();
  }

  onNewAccountUpdate = () => {
  }

  retrieveAccounts () {
    if (!this._isMounted) {
      return;
    }

    const api = this.context.api;
    let accounts = [];
    const contracts = {};
    const tokens = [];

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo(),
        api.ethcore.registryAddress()
      ])
      .then(([addresses, infos, registryAddress]) => {
        accounts = addresses
          .filter((address) => infos[address].uuid)
          .map((address) => {
            const info = infos[address];

            return {
              address: address,
              name: info.name,
              uuid: info.uuid,
              meta: info.meta
            };
          });

        contracts.registry = api.newContract(registryAbi).at(registryAddress);

        return contracts.registry
          .getAddress
          .call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        contracts.tokenreg = api.newContract(tokenRegAbi).at(tokenregAddress);

        return contracts.tokenreg
          .tokenCount
          .call();
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
          console.log(token[0], token[1], token[2].toFormat(), token[3]);

          const contract = api.newContract(eip20Abi).at(token[0]);

          tokens.push({
            address: token[0],
            image: `images/tokens/${token[3].toLowerCase()}-32x32.png`,
            supply: '0',
            token: token[1],
            type: token[3],
            contract
          });

          return contract.totalSupply.call();
        }));
      })
      .then((supplies) => {
        console.log('supplies', supplies.map((supply) => supply.toFormat()));

        supplies.forEach((supply, idx) => {
          tokens[idx].supply = supply.toString();
        });

        this.setState({
          accounts,
          contracts,
          tokens
        });

        setTimeout(() => this.retrieveAccounts(), 2500);
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
