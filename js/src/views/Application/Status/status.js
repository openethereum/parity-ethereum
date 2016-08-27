import React, { Component, PropTypes } from 'react';

import styles from './style.css';

export default class Status extends Component {
  static propTypes = {
    blockNumber: PropTypes.object,
    clientVersion: PropTypes.string,
    peerCount: PropTypes.object
  }

  render () {
    const { clientVersion, blockNumber, peerCount } = this.props;

    return (
      <div className={ styles.status }>
        <div className={ styles.version }>{ clientVersion }</div>
        <div> className={ styles.block }{ blockNumber.toFormat() } blocks</div>
        <div className={ styles.peers }>{ peerCount.toFormat() } peers</div>
      </div>
    );
  }
}
