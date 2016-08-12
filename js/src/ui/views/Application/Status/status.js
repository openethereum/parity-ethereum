import React, { Component } from 'react';

import styles from './style.css';

export default class Status extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  state = {
    clientVersion: '',
    peerCount: 0,
    blockNumber: 0,
    syncing: false
  }

  componentWillMount () {
    this.poll();
  }

  render () {
    return (
      <div className={ styles.status }>
        <div>{ this.state.clientVersion }</div>
        <div>{ this.state.peerCount } peers</div>
        <div>{ this.state.blockNumber }</div>
      </div>
    );
  }

  poll () {
    const api = this.context.api;

    Promise
      .all([
        api.web3.clientVersion(),
        api.net.peerCount(),
        api.eth.blockNumber(),
        api.eth.syncing()
      ])
      .then(([clientVersion, peerCount, blockNumber, syncing]) => {
        this.setState({
          blockNumber: blockNumber.toFormat(0),
          clientVersion: clientVersion,
          peerCount: peerCount.toString(),
          syncing: syncing
        });
      });

    setTimeout(() => this.poll(), 2500);
  }
}
