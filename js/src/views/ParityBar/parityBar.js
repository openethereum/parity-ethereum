import React, { Component } from 'react';

import { IconButton } from 'material-ui';
import ActionSwapVert from 'material-ui/svg-icons/action/swap-vert';

import styles from './style.css';

export default class ParityBar extends Component {
  render () {
    return (
      <div className={ styles.bar }>
        <div className={ styles.corner }>
          <a
            className={ styles.noshow }
            href='/#/apps'>
            <img
              className={ styles.logo }
              src='/images/paritybar.png' />
          </a>
          <IconButton className={ styles.button }>
            <ActionSwapVert />
          </IconButton>
        </div>
      </div>
    );
  }
}
