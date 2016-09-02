import React, { Component, PropTypes } from 'react';

import styles from './dapp.css';

export default class Dapp extends Component {
  static propTypes = {
    params: PropTypes.object
  };

  render () {
    const { name } = this.props.params;
    const src = `${name}.html`;

    return (
      <iframe
        className={ styles.frame }
        frameBorder={ 0 }
        name={ name }
        sandbox='allow-scripts'
        scrolling='auto'
        src={ src }>
      </iframe>
    );
  }
}
