import React, { Component, PropTypes } from 'react';

import LinearProgress from 'material-ui/LinearProgress';

import styles from '../style.css';

export default class Complete extends Component {
  static propTypes = {
    txhash: PropTypes.string,
    sending: PropTypes.bool
  }

  render () {
    if (this.props.sending) {
      return (
        <div>
          <div className={ styles.info }>
            The transaction is sending, please wait until the transaction hash is received
          </div>
          <LinearProgress mode='indeterminate' />
        </div>
      );
    }

    const txlink = `https://etherscan.io/tx/${this.props.txhash}`;

    return (
      <div>
        <div className={ styles.info }>
          The transaction was send with a transaction hash (useful for tracking on a block explorer) of
        </div>
        <div>
          <a href={ txlink } target='_blank'>{ this.props.txhash }</a>
        </div>
      </div>
    );
  }
}
