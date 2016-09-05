import React, { Component, PropTypes } from 'react';

import styles from './accountDetailsGeth.css';

export default class AccountDetailsGeth extends Component {
  static propTypes = {
    addresses: PropTypes.array
  }

  render () {
    const { addresses } = this.props;

    const formatted = addresses.map((address, idx) => {
      const comma = !idx ? '' : ((idx === addresses.length - 1) ? ' & ' : ', ');
      return `${comma}${address}`;
    }).join('');

    return (
      <div className={ styles.details }>
        <div>You have imported { addresses.length } addresses from the Geth keystore:</div>
        <div className={ styles.address }>{ formatted }</div>
      </div>
    );
  }
}
