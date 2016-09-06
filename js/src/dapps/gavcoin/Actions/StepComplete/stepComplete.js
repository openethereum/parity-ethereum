import React, { Component } from 'react';

import styles from '../actions.css';

export default class StepComplete extends Component {
  render () {
    return (
      <div className={ styles.dialogtext }>
        Your transaction has been posted. Please visit the <a href='http://127.0.0.1:8080/#/signer' className={ styles.link } target='_blank'>Parity Signer</a> to authenticate the transfer.
      </div>
    );
  }
}
