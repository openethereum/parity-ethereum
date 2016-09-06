import React, { Component } from 'react';

import styles from './frameError.css';

export default class FrameError extends Component {
  render () {
    return (
      <div className={ styles.error }>
        ERROR: This application cannot and should not be loaded in an embedded iFrame
      </div>
    );
  }
}
