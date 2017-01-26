// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { nullableProptype } from '~/util/proptypes';

import styles from './application.css';
import Accounts from '../Accounts';
import Events from '../Events';
import Lookup from '../Lookup';
import Names from '../Names';
import Records from '../Records';
import Reverse from '../Reverse';

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object.isRequired,
    api: PropTypes.object.isRequired
  };

  getChildContext () {
    return { muiTheme, api: window.parity.api };
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contract: nullableProptype(PropTypes.object.isRequired),
    fee: nullableProptype(PropTypes.object.isRequired)
  };

  render () {
    const { api } = window.parity;
    const { contract, fee } = this.props;
    let warning = null;

    return (
      <div>
        { warning }
        <div className={ styles.header }>
          <h1>RΞgistry</h1>
          <Accounts />
        </div>
        { contract && fee ? (
          <div>
            <Lookup />
            { this.renderActions() }
            <Events />
            <div className={ styles.warning }>
              WARNING: The name registry is experimental. Please ensure that you understand the risks, benefits & consequences of registering a name before doing so. A non-refundable fee of { api.util.fromWei(fee).toFormat(3) }<small>ETH</small> is required for all registrations.
            </div>
          </div>
        ) : (
          <CircularProgress size={ 60 } />
        ) }
      </div>
    );
  }

  renderActions () {
    const hasAccount = !!this.props.accounts.selected;

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
        <Names />
        <Records />
        <Reverse />
      </div>
    );
  }
}
