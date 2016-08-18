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
        return (
          <div
            className={ styles.balance }
            key={ balance.token }>
            <img
              src={ balance.img }
              alt={ balance.type } />
            <div>
              { Api.format.fromWei(balance.value).toFormat() } { balance.token }
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

    Promise
      .all([
        api.eth.getBalance(this.props.address)
      ])
      .then(([balance]) => {
        this.setState({
          balances: [
            {
              token: 'ΞTH',
              value: balance.toString(),
              img: 'images/tokens/ethereum-32x32.png',
              type: 'Ethereum'
            }
          ]
        }, this.updateParent);
      });
  }
}
