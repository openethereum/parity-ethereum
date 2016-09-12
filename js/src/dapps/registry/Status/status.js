import React, { Component, PropTypes } from 'react';

import styles from './status.css';

export default class Status extends Component {

  static propTypes = {
    address: PropTypes.string,
    owner: PropTypes.string
  }

  render () {
    const { address, owner } = this.props;

    return (
      <div className={ styles.status }>
        <div className={ styles.address }>Registry at { address }</div>
        <div className={ styles.owner }>Owned by { owner }</div>
      </div>
    );
  }
}
