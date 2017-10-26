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

import { api } from '../parity';

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
    isLoading: PropTypes.bool.isRequired,

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
          fee={ contract.fee }
        />

        <Actions />

        <Tokens />
        <div className={ styles.warning }>
          WARNING: The token registry is experimental. Please ensure that you understand the steps, risks, benefits & consequences of registering a token before doing so. A non-refundable fee of { api.util.fromWei(contract.fee).toFormat(3) }<small>ETH</small> is required for all registrations.
        </div>
      </div>
    );
  }

  getChildContext () {
    return {
      muiTheme
    };
  }
}
