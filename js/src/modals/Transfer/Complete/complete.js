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

import LinearProgress from 'material-ui/LinearProgress';

import styles from '../transfer.css';

export default class Complete extends Component {
  static propTypes = {
    txhash: PropTypes.string,
    sending: PropTypes.bool
  }

  render () {
    const { sending } = this.props;

    if (sending) {
      return (
        <div>
          <div className={ styles.info }>
            The transaction is sending, please wait until the transaction hash is received
          </div>
          <LinearProgress mode='indeterminate' />
        </div>
      );
    }

    return (
      <div>
        <div className={ styles.info }>
          The transaction was sent and awaits verification in the signer. <a href='/#/signer'>Enter the signer</a> and authenticate the correct transactions with your account password.
        </div>
      </div>
    );
  }
}
