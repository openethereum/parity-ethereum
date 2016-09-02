import React, { Component, PropTypes } from 'react';

import LinearProgress from 'material-ui/LinearProgress';

import styles from '../style.css';

export default class Complete extends Component {
  static propTypes = {
    txhash: PropTypes.string,
    sending: PropTypes.bool
  }

  render () {
    const { sending } = this.props;

    if (sending) {
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
          The transaction was sent and awaits verification in the signer. <a href='/#/signer'>Enter the signer</a> and authenticate the correct transactions with your account password.
        </div>
      </div>
    );
  }
}
