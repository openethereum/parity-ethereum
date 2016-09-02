import React, { Component } from 'react';

import styles from '../application.css';

export default class FrameError extends Component {
  render () {
    return (
      <div className={ styles.apperror }>
        ERROR: This application cannot and should not be loaded in an embedded iFrame
      </div>
    );
  }
}
