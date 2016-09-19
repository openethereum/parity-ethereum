import React, { Component, PropTypes } from 'react';
import CircularProgress from 'material-ui/CircularProgress';

import styles from './loading.css';

export default class Loading extends Component {
  static propTypes = {
    size: PropTypes.number
  };

  render () {
    return (
      <div className={ styles.loading }>
        <CircularProgress size={ this.props.size || 2 } />
      </div>
    );
  }
}
