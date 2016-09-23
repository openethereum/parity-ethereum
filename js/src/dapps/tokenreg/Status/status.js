import React, { Component, PropTypes } from 'react';

import Chip from '../Chip';

import styles from './status.css';

const { api } = window.parity;

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    fee: PropTypes.object
  };

  render () {
    const { address, fee } = this.props;

    return (
      <div className={ styles.status }>
        <h1 className={ styles.title }>Token Registry</h1>

        <Chip
          isAddress
          value={ address }
          label='Address' />

        <Chip
          isAddress={ false }
          value={ api.util.fromWei(fee).toFixed(3) + 'ΞTH' }
          label='Fee' />
      </div>
    );
  }
}
