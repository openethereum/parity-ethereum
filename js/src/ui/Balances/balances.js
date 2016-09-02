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
    let body = this.props.account.balances
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
              src={ token.images.small }
              alt={ token.name } />
            <div>{ value }<small> { token.tag }</small></div>
          </div>
        );
      });

    if (!body.length) {
      body = (
        <div className={ styles.empty }>
          There are no balances associated with this account
        </div>
      );
    }

    return (
      <div className={ styles.balances }>
        { body }
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
