import React, { Component, PropTypes } from 'react';

import styles from './page.css';

export default class Page extends Component {
  static propTypes = {
    className: PropTypes.string,
    children: PropTypes.node
  };

  render () {
    const { className, children } = this.props;
    const classes = `${styles.layout} ${className}`;

    return (
      <div className={ classes }>
        { children }
      </div>
    );
  }
}
