import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Api from '../../api';

import styles from './style.css';

export default class Balances extends Component {
  static propTypes = {
    account: PropTypes.object,
    onChange: PropTypes.func
  }

  render () {
    if (!this.props.account) {
      return null;
    }

    const balances = this.props.account.balances
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const token = balance.token;
        const value = token.format
          ? new BigNumber(balance.value).div(new BigNumber(token.format)).toFormat(3)
          : Api.format.fromWei(balance.value).toFormat(3);

        return (
          <div
            className={ styles.balance }
            key={ token.tag }>
            <img
              src={ token.image }
              alt={ token.type } />
            <div>{ value } { token.tag }</div>
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
