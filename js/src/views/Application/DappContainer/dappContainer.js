import React, { Component, PropTypes } from 'react';

import ParityBar from '../../ParityBar';

import styles from '../application.css';

export default class DappContainer extends Component {
  static propTypes = {
    children: PropTypes.node
  };

  render () {
    const { children } = this.props;

    return (
      <div className={ styles.container }>
        { children }
        <ParityBar />
      </div>
    );
  }
}
