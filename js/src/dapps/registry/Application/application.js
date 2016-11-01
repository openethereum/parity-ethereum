// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.
import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';
const muiTheme = getMuiTheme(lightBaseTheme);

import CircularProgress from 'material-ui/CircularProgress';
import { Card, CardText } from 'material-ui/Card';
import styles from './application.css';
import Accounts from '../Accounts';
import Events from '../Events';
import Lookup from '../Lookup';
import Names from '../Names';
import Records from '../Records';

const nullable = (type) => React.PropTypes.oneOfType([ React.PropTypes.oneOf([ null ]), type ]);

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object.isRequired,
    api: PropTypes.object.isRequired
  };

  getChildContext () {
    return { muiTheme, api: window.parity.api };
  }

  static propTypes = {
    actions: PropTypes.object.isRequired,
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    contract: nullable(PropTypes.object.isRequired),
    fee: nullable(PropTypes.object.isRequired),
    lookup: PropTypes.object.isRequired,
    events: PropTypes.object.isRequired,
    names: PropTypes.object.isRequired,
    records: PropTypes.object.isRequired
  };

  render () {
    const {
      actions,
      accounts, contacts,
      contract, fee,
      lookup,
      events
    } = this.props;

    return (
      <div>
        <div className={ styles.header }>
          <h1>RΞgistry</h1>
          <Accounts { ...accounts } actions={ actions.accounts } />
        </div>
        { contract && fee ? (
          <div>
            <Lookup { ...lookup } accounts={ accounts.all } contacts={ contacts } actions={ actions.lookup } />

            { this.renderActions() }

            <Events { ...events } accounts={ accounts.all } contacts={ contacts } actions={ actions.events } />
            <p className={ styles.address }>
              The Registry is provided by the contract at <code>{ contract.address }.</code>
            </p>
          </div>
        ) : (
          <CircularProgress size={ 60 } />
        ) }
      </div>
    );
  }

  renderActions () {
    const {
      actions,
      accounts,
      fee,
      names,
      records
    } = this.props;

    const hasAccount = !!accounts.selected;

    if (!hasAccount) {
      return (
        <Card className={ styles.actions }>
          <CardText>
            Please select a valid account in order
            to execute actions.
          </CardText>
        </Card>
      );
    }

    return (
      <div>
        <Names { ...names } fee={ fee } actions={ actions.names } />
        <Records { ...records } actions={ actions.records } />
      </div>
    );
  }

}
