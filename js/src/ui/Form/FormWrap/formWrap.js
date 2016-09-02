import React, { Component, PropTypes } from 'react';

import styles from './formWrap.css';

export default class FormWrap extends Component {
  static propTypes = {
    children: PropTypes.node
  }

  render () {
    return (
      <div className={ styles.stretch }>
        { this.props.children }
      </div>
    );
  }
}
