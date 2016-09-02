import React, { Component } from 'react';

import styles from './loading.css';

export default class Loading extends Component {
  render () {
    return (
      <div className={ styles.loading }>
        Loading ...
      </div>
    );
  }
}
