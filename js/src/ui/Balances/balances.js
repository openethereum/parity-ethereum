import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Api from '../../api';

import styles from './style.css';

export default class Balances extends Component {
  static contextTypes = {
    api: PropTypes.object,
    tokens: PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    onChange: PropTypes.func
  }

  state = {
    balances: []
  }

  componentDidMount () {
    this.getBalances();
  }

  render () {
    const balances = this.state.balances
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const value = balance.format
          ? new BigNumber(balance.value).div(new BigNumber(balance.format)).toFormat(3)
          : Api.format.fromWei(balance.value).toFormat(3);
        return (
          <div
            className={ styles.balance }
            key={ balance.token }>
            <div>
              { value } { balance.token }
            </div>
            <img
              src={ balance.image }
              alt={ balance.type } />
          </div>
        );
      });

    if (!balances.length) {
      return null;
    }

    return (
      <div className={ styles.balances }>
        { balances }
      </div>
    );
  }

  updateParent = () => {
    if (!this.props.onChange) {
      return;
    }

    this.props.onChange(this.state.balances);
  }

  getBalances = () => {
    const api = this.context.api;
    const calls = this.context.tokens.map((token) => token.contract.balanceOf.call({}, [this.props.address]));

    api.eth
      .getBalance(this.props.address)
      .then((balance) => {
        setTimeout(this.getBalances, 2500);

        const balances = [{
          image: 'images/tokens/ethereum-32x32.png',
          token: 'ΞTH',
          type: 'Ethereum',
          value: balance.toString()
        }];

        return Promise
          .all(calls)
          .then((tokenBalances) => {
            if (tokenBalances && tokenBalances.length) {
              tokenBalances.forEach((balance, idx) => {
                const token = this.context.tokens[idx];

                if (token) {
                  balances.push({
                    format: token.format,
                    image: token.image,
                    token: token.token,
                    type: token.type,
                    value: balance.toString()
                  });
                }
              });
            }

            this.setState({
              balances
            }, this.updateParent);
          });
      });
  }
}
