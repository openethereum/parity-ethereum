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
import { FormattedMessage } from 'react-intl';

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags } from '~/ui';
import CopyToClipboard from '~/ui/CopyToClipboard';
import Certifications from '~/ui/Certifications';

import styles from './header.css';

export default class Header extends Component {
  static propTypes = {
    account: PropTypes.object,
    balance: PropTypes.object,
    children: PropTypes.node,
    className: PropTypes.string,
    hideName: PropTypes.bool,
    isContract: PropTypes.bool
  };

  static defaultProps = {
    children: null,
    className: '',
    hideName: false,
    isContract: false
  };

  render () {
    const { account, balance, children, className, hideName } = this.props;

    if (!account) {
      return null;
    }

    const { address } = account;
    const meta = account.meta || {};

    return (
      <div className={ className }>
        <Container>
          <IdentityIcon address={ address } />
          <div className={ styles.floatleft }>
            { this.renderName() }
            <div className={ [ hideName ? styles.bigaddress : '', styles.addressline ].join(' ') }>
              <CopyToClipboard data={ address } />
              <div className={ styles.address }>{ address }</div>
            </div>
            { this.renderUuid() }
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
              balance={ balance }
            />
            <Certifications address={ address } />
          </div>
          { children }
        </Container>
      </div>
    );
  }

  renderName () {
    const { hideName } = this.props;

    if (hideName) {
      return null;
    }

    const { address } = this.props.account;

    return (
      <ContainerTitle
        title={
          <IdentityName
            address={ address }
            unknown
          />
        }
      />
    );
  }

  renderTxCount () {
    const { balance, isContract } = this.props;

    if (!balance || isContract) {
      return null;
    }

    const { txCount } = balance;

    if (!txCount) {
      return null;
    }

    return (
      <div className={ styles.infoline }>
        <FormattedMessage
          id='account.header.outgoingTransactions'
          defaultMessage='{count} outgoing transactions'
          values={ {
            count: txCount.toFormat()
          } }
        />
      </div>
    );
  }

  renderUuid () {
    const { uuid } = this.props.account;

    if (!uuid) {
      return null;
    }

    return (
      <div className={ styles.uuidline }>
        <FormattedMessage
          id='account.header.uuid'
          defaultMessage='uuid: {uuid}'
          values={ {
            uuid
          } }
        />
      </div>
    );
  }
}
