import React, { Component, PropTypes } from 'react';

import Api from '../../api';

import styles from './style.css';

export default class Balances extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    address: PropTypes.string.isRequired
  }

  state = {
    balances: []
  }

  componentWillMount () {
    this.getBalances();
  }

  render () {
    const balances = this.state.balances
      .filter((balance) => balance.value.gt(0))
      .map((balance) => {
        return (
          <div
            className={ styles.balance }
            key={ balance.token }>
            <img
              src={ balance.img }
              alt={ balance.type } />
            <div>
              { balance.value.toFormat(8) } { balance.token }
            </div>
          </div>
        );
      });

    return (
      <div>{ balances }</div>
    );
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
              value: Api.format.fromWei(balance),
              img: 'images/ethereum-32x32.png',
              type: 'Ethereum'
            }
          ]
        });
      });
  }
}
