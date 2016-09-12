import React, { Component, PropTypes } from 'react';

import styles from './apiProvider.css';

export default class ApiProvider extends Component {
  static propTypes = {
    api: PropTypes.object.isRequired,
    children: PropTypes.node.isRequired
  }

  static childContextTypes = {
    api: PropTypes.object
  }

  render () {
    const { children } = this.props;

    return (
      <div className={ styles.api }>{ children }</div>
    );
  }

  getChildContext () {
    const { api } = this.props;

    return {
      api
    };
  }
}
