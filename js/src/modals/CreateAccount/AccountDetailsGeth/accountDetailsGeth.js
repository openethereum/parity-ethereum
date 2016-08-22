import React, { Component, PropTypes } from 'react';

import styles from './style.css';

export default class AccountDetailsGeth extends Component {
  static propTypes = {
    addresses: PropTypes.array
  }

  render () {
    const addresses = this.props.addresses.map((address, idx) => {
      const comma = !idx ? '' : ((idx === this.props.addresses.length - 1) ? ' & ' : ', ');
      return `${comma}${address}`;
    }).join('');

    return (
      <div className={ styles.details }>
        <div>You have imported { this.props.addresses.length } addresses from the Geth keystore:</div>
        <div className={ styles.address }>{ addresses }</div>
      </div>
    );
  }
}
