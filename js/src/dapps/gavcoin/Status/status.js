import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins, formatEth } from '../format';

import styles from './style.css';

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    gavBalance: PropTypes.object,
    blockNumber: PropTypes.object,
    totalSupply: PropTypes.object,
    remaining: PropTypes.object,
    price: PropTypes.object,
    children: PropTypes.node
  }

  render () {
    const { blockNumber, gavBalance, totalSupply, remaining, price } = this.props;

    if (!totalSupply) {
      return null;
    }

    return (
      <div className={ styles.status }>
        <div className={ styles.item }>
          <div className={ styles.heading }>&nbsp;</div>
          <div className={ styles.hero }>
            { formatCoins(remaining, -1) }
          </div>
          <div className={ styles.byline }>
            available for { formatEth(price) }ΞTH
          </div>
        </div>
        <div className={ styles.item }>
          <div className={ styles.heading }>GAVcoin</div>
          <div className={ styles.hero }>
            { formatCoins(totalSupply, -1) }
          </div>
          <div className={ styles.byline }>
            total at { formatBlockNumber(blockNumber) }
          </div>
        </div>
        <div className={ styles.item }>
          <div className={ styles.heading }>&nbsp;</div>
          <div className={ styles.hero }>
            { formatCoins(gavBalance, -1) }
          </div>
          <div className={ styles.byline }>
            coin balance
          </div>
        </div>
        { this.props.children }
      </div>
    );
  }
}
