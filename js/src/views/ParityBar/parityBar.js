import React, { Component, PropTypes } from 'react';

import Toolbar from 'material-ui/Toolbar';

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
      <div
        className={ styles.bar }>
        <div>
          <img
            className={ styles.logo }
            src='images/paritybar.png' />
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
