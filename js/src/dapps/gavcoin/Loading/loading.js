import React, { Component } from 'react';

import { CircularProgress } from 'material-ui';

import styles from './style.css';

export default class Loading extends Component {
  render () {
    return (
      <div className={ styles.loading }>
        <CircularProgress size={ 2 } />
      </div>
    );
  }
}
