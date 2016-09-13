import React, { Component, PropTypes } from 'react';

import styles from './badge.css';

export default class Badge extends Component {
  static propTypes = {
    className: PropTypes.string,
    color: PropTypes.string,
    value: PropTypes.any
  };

  render () {
    const { className, color, value } = this.props;
    const classes = `${styles.bubble} ${styles[color || 'default']} ${className}`;

    return (
      <div className={ classes }>
        { value }
      </div>
    );
  }
}
