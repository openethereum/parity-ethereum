import React, { Component, PropTypes } from 'react';

import { IconButton } from 'material-ui';
import ActionSwapVert from 'material-ui/svg-icons/action/swap-vert';

import Api from '../../api';
import muiTheme from '../../ui/Theme';

import styles from './style.css';

const api = new Api(new Api.Transport.Http('/rpc/'));

export default class ParityBar extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    muiTheme: PropTypes.object
  }

  render () {
    return (
      <div className={ styles.bar }>
        <div className={ styles.corner }>
          <a
            className={ styles.noshow }
            href='/'>
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

  getChildContext () {
    return {
      api,
      muiTheme
    };
  }
}
