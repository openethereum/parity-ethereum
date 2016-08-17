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

    return (
      <div>
        <div className={ styles.info }>
          The transaction was sent and awaits verification in the signer. <a href='http://127.0.0.1:8180' target='_blank'>Enter the signer</a> and authenticate the correct transactions with your account password.
        </div>
      </div>
    );
  }
}
