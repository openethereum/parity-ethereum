import React, { Component } from 'react';

import AppBar from 'material-ui/AppBar';
import { isExtension } from '../../utils/extension';

import styles from './Header.css';

export default class Header extends Component {

  title = 'Parity Trusted Signer';
  styles = {
    backgroundColor: isExtension() ? '#6691C2' : '#FF5722'
  };

  render () {
    return (
      <AppBar
        title={ this.title }
        className={ styles.bar }
        style={ this.styles }
        showMenuIconButton={ false }
      />
    );
  }

}
