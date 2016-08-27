import React, { Component, PropTypes } from 'react';

import styles from './style.css';

export default class Dapp extends Component {
  static propTypes = {
    params: PropTypes.object
  };

  render () {
    const { name } = this.props.params;
    const src = `dapps/${name}.html`;

    return (
      <iframe
        className={ styles.frame }
        src={ src }>
      </iframe>
    );
  }
}
