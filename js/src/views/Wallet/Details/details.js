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
import moment from 'moment';

import { Container, InputAddress, Input } from '../../../ui';

import styles from '../wallet.css';

export default class WalletDetails extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    owners: PropTypes.array,
    require: PropTypes.object,
    dailylimit: PropTypes.object
  };

  render () {
    return (
      <div className={ styles.details }>
        <Container title='Owners'>
          { this.renderOwners() }
        </Container>

        <Container title='Details'>
          { this.renderDetails() }
        </Container>
      </div>
    );
  }

  renderOwners () {
    const { owners } = this.props;

    if (!owners) {
      return null;
    }

    const ownersList = owners.map((address) => (
      <InputAddress
        key={ address }
        value={ address }
        disabled
        text
      />
    ));

    return (
      <div>
        { ownersList }
      </div>
    );
  }

  renderDetails () {
    const { require, dailylimit } = this.props;
    const { api } = this.context;

    if (!dailylimit || !dailylimit.limit) {
      return null;
    }

    const limit = api.util.fromWei(dailylimit.limit).toFormat(3);
    const spent = api.util.fromWei(dailylimit.spent).toFormat(3);
    const date = moment(dailylimit.last.toNumber() * 24 * 3600 * 1000);

    return (
      <div>
        <p>
          <span>This wallet requires at least</span>
          <span className={ styles.detail }>{ require.toFormat() } owners</span>
          <span>to validate any action (transactions, modifications).</span>
        </p>

        <p>
          <span className={ styles.detail }>{ spent }<span className={ styles.eth } /></span>
          <span>has been spent today, out of</span>
          <span className={ styles.detail }>{ limit }<span className={ styles.eth } /></span>
          <span>set as the daily limit, which has been reset on</span>
          <span className={ styles.detail }>{ date.format('LL') }</span>
        </p>
      </div>
    );
  }
}
