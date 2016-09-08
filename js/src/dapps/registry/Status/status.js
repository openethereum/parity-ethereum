import React, { Component, PropTypes } from 'react';

import styles from './status.css';

const { api } = window.parity;

export default class Status extends Component {

  static propTypes = {
    address: PropTypes.string,
    fee: PropTypes.object,
    owner: PropTypes.string
  }

  render () {
    const { address, fee, owner } = this.props;

    return (
      <div className={ styles.status }>
        <div className={ styles.address }>Registry at { address }</div>
        <div className={ styles.owner }>Owned by { owner }</div>
        <div className={ styles.fee }>Registration fee { api.format.fromWei(fee).toFixed(3) }ΞTH</div>
      </div>
    );
  }
}
