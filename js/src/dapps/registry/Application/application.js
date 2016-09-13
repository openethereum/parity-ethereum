import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';
const muiTheme = getMuiTheme(lightBaseTheme);

import CircularProgress from 'material-ui/CircularProgress';
import styles from './application.css';
import Accounts from '../accounts';
import Lookup from '../lookup';
import Register from '../register';
import Events from '../events';
import Status from '../status';

export default class Application extends Component {
  static childContextTypes = { muiTheme: PropTypes.object };
  getChildContext () {
    return { muiTheme };
  }

  static propTypes = {
    actions: PropTypes.object,
    accounts: PropTypes.object,
    account: PropTypes.object,
    contract: PropTypes.object,
    owner: PropTypes.string,
    fee: PropTypes.object,
    lookup: PropTypes.object,
    events: PropTypes.array
  };

  render () {
    const {
      actions,
      accounts,
      contract, fee, owner,
      lookup,
      events,
      register
    } = this.props;

    return (
      <div>
        <div className={ styles.header }>
          <h1>RΞgistry</h1>
          <Accounts all={ accounts.all } selected={ accounts.selected } actions={ actions.accounts } />
        </div>
        { contract && fee && owner
          ? (
            <div>
              <Lookup lookup={ lookup } actions={ actions.lookup } />
              <Register register={ register } fee={ fee } actions={ actions.register } />
              <Events events={ events } actions={ actions.events } />
              <Status address={ contract.address } owner={ owner } />
            </div>
          ) : <CircularProgress size={ 1 } />
        }
      </div>
    );
  }

}
