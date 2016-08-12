import React, { Component, PropTypes } from 'react';

import { Card } from 'material-ui/Card';

import styles from './style.css';

export default class Container extends Component {
  static propTypes = {
    children: PropTypes.node
  }

  render () {
    return (
      <div className={ styles.padded }>
        <Card>
          { this.props.children }
        </Card>
      </div>
    );
  }
}
