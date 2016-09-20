import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';

import Loading from '../Loading';
import Status from '../Status';
import Tokens from '../Tokens';
import Actions from '../Actions';

import styles from './application.css';

const muiTheme = getMuiTheme({
  palette: {
    primary1Color: '#27ae60'
  }
});

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object
  }

  static propTypes = {
    isLoading: PropTypes.bool,
    contract: PropTypes.object
  };

  render () {
    const { isLoading, contract } = this.props;

    if (isLoading) {
      return (
        <Loading />
      );
    }

    return (
      <div className={ styles.application }>
        <Status
          address={ contract.address }
          fee={ contract.fee } />

        <Actions />

        <Tokens />
      </div>
    );
  }

  getChildContext () {
    return {
      muiTheme
    };
  }

}
