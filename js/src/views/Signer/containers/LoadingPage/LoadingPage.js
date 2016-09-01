import React, { Component } from 'react';

import styles from './LoadingPage.css';

export default class LoadingPage extends Component {
  render () {
    return (
      <div className={ styles.main }>
        <h2>Connecting to Parity...</h2>
      </div>
    );
  }
}
