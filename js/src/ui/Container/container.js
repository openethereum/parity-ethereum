import React, { Component, PropTypes } from 'react';

import { Card } from 'material-ui/Card';

import styles from './container.css';

export default class Container extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string
  }

  render () {
    const { children, className } = this.props;
    const classes = `${styles.container} ${className}`;

    return (
      <div className={ classes }>
        <Card className={ styles.padded }>
          { children }
        </Card>
      </div>
    );
  }
}
