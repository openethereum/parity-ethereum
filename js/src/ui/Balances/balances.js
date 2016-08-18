import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Api from '../../api';

import styles from './style.css';

export default class Balances extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    tokens: PropTypes.array,
    onChange: PropTypes.func
  }

  state = {
    balances: []
  }

  componentWillMount () {
    this.getBalances();
  }

  render () {
    const balances = this.state.balances
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const value = balance.format
          ? Api.format.fromWei(balance.value).toFormat()
          : new BigNumber(balance.value).toFormat();
        return (
          <div
            className={ styles.balance }
            key={ balance.token }>
            <img
              src={ balance.image }
              alt={ balance.type } />
            <div>
              { value } { balance.token }
            </div>
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

  getBalances () {
    const api = this.context.api;
    const calls = (this.props.tokens || []).map((token) => token.contract.balanceOf.call({}, [this.props.address]));

    api.eth
      .getBalance(this.props.address)
      .then((balance) => {
        const balances = [{
          format: true,
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
                const token = this.props.tokens[idx];

                if (token) {
                  balances.push({
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
