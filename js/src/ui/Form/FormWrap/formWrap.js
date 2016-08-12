import React, { Component, PropTypes } from 'react';

import styles from './style.css';

export default class FormWrap extends Component {
  static propTypes = {
    children: PropTypes.object
  }

  render () {
    return (
      <div className={ styles.stretch }>
        { this.props.children }
      </div>
    );
  }
}
