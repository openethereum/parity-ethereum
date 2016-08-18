import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Api from '../../api';

import styles from './style.css';

export default class Balances extends Component {
  static contextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array,
    tokens: PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    onChange: PropTypes.func
  }

  render () {
    const account = this.context.accounts.find((acc) => acc.address === this.props.address);

    if (!account) {
      return null;
    }

    const balances = account.balances
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const token = balance.token;
        const value = token.format
          ? new BigNumber(balance.value).div(new BigNumber(token.format)).toFormat(3)
          : Api.format.fromWei(balance.value).toFormat(3);

        return (
          <div
            className={ styles.balance }
            key={ token.token }>
            <div>
              { value } { token.token }
            </div>
            <img
              src={ token.image }
              alt={ token.type } />
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
}
