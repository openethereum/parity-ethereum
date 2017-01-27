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

import { Container, InputAddress } from '~/ui';

import styles from '../wallet.css';

export default class WalletDetails extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    owners: PropTypes.array,
    require: PropTypes.object,
    className: PropTypes.string
  };

  static defaultProps = {
    className: ''
  };

  render () {
    const { className } = this.props;

    return (
      <div className={ [ styles.details, className ].join(' ') }>
        <Container title='Details'>
          { this.renderDetails() }
          { this.renderOwners() }
        </Container>
      </div>
    );
  }

  renderOwners () {
    const { owners } = this.props;

    if (!owners) {
      return null;
    }

    const ownersList = owners.map((owner, idx) => {
      const address = typeof owner === 'object'
        ? owner.address
        : owner;

      return (
        <InputAddress
          key={ `${idx}_${address}` }
          value={ address }
          disabled
          text
        />
      );
    });

    return (
      <div>
        { ownersList }
      </div>
    );
  }

  renderDetails () {
    const { require } = this.props;

    if (!require) {
      return null;
    }

    return (
      <div>
        <p>
          <span>This wallet requires at least</span>
          <span className={ styles.detail }>{ require.toFormat() } owners</span>
          <span>to validate any action (transactions, modifications).</span>
        </p>
      </div>
    );
  }
}
