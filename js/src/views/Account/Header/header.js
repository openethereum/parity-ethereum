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

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags } from '~/ui';
import CopyToClipboard from '~/ui/CopyToClipboard';
import Certifications from '~/ui/Certifications';

import styles from './header.css';

export default class Header extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    account: PropTypes.object,
    balance: PropTypes.object,
    className: PropTypes.string,
    children: PropTypes.node
  };

  static defaultProps = {
    className: '',
    children: null
  };

  render () {
    const { api } = this.context;
    const { account, balance, className, children } = this.props;
    const { address, meta, uuid } = account;

    if (!account) {
      return null;
    }

    const uuidText = !uuid
      ? null
      : <div className={ styles.uuidline }>uuid: { uuid }</div>;

    return (
      <div className={ className }>
        <Container>
          <IdentityIcon
            address={ address } />
          <div className={ styles.floatleft }>
            <ContainerTitle title={ <IdentityName address={ address } unknown /> } />
            <div className={ styles.addressline }>
              <CopyToClipboard data={ address } />
              <div className={ styles.address }>{ address }</div>
            </div>
            { uuidText }
            <div className={ styles.infoline }>
              { meta.description }
            </div>
            { this.renderTxCount() }
          </div>
          <div className={ styles.tags }>
            <Tags tags={ meta.tags } />
          </div>
          <div className={ styles.balances }>
            <Balance
              account={ account }
              balance={ balance } />
            <Certifications
              account={ account.address }
              dappsUrl={ api.dappsUrl }
            />
          </div>
          { children }
        </Container>
      </div>
    );
  }

  renderTxCount () {
    const { balance } = this.props;

    if (!balance) {
      return null;
    }

    const { txCount } = balance;

    if (!txCount) {
      return null;
    }

    return (
      <div className={ styles.infoline }>
        { txCount.toFormat() } outgoing transactions
      </div>
    );
  }
}
