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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { Balance, Certifications, Container, CopyToClipboard, ContainerTitle, IdentityIcon, IdentityName, QrCode, Tags, VaultTag } from '@parity/ui';

import styles from './header.css';

export default class Header extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    account: PropTypes.object,
    children: PropTypes.node,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    hideName: PropTypes.bool,
    isContract: PropTypes.bool
  };

  static defaultProps = {
    children: null,
    className: '',
    hideName: false,
    isContract: false
  };

  state = {
    txCount: null
  };

  txCountSubId = null;

  componentWillMount () {
    if (this.props.account && !this.props.isContract) {
      this.subscribeTxCount();
    }
  }

  componentWillUnmount () {
    this.unsubscribeTxCount();
  }

  subscribeTxCount () {
    const { api } = this.context;

    api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          return console.error(error);
        }

        api.eth.getTransactionCount(this.props.account.address)
          .then((txCount) => this.setState({ txCount }));
      })
      .then((subscriptionId) => {
        this.txCountSubId = subscriptionId;
      });
  }

  unsubscribeTxCount () {
    if (!this.txCountSubId) {
      return;
    }

    this.context.api.unsubscribe(this.txCountSubId);
  }

  render () {
    const { account, children, className, disabled, hideName } = this.props;

    if (!account) {
      return null;
    }

    const { address } = account;
    const meta = account.meta || {};

    return (
      <div className={ className }>
        <Container>
          <QrCode
            className={ styles.qrcode }
            value={ address }
          />
          <IdentityIcon
            address={ address }
            className={ styles.identityIcon }
            disabled={ disabled }
          />
          <div className={ styles.info }>
            { this.renderName() }
            <div className={ [ hideName ? styles.bigaddress : '', styles.addressline ].join(' ') }>
              <CopyToClipboard data={ address } />
              <div className={ styles.address }>
                { address }
              </div>
            </div>
            { this.renderUuid() }
            <div className={ styles.infoline }>
              { meta.description }
            </div>
            { this.renderTxCount() }
            <div className={ styles.balances }>
              <Balance
                address={ address }
              />
              <Certifications address={ address } />
              { this.renderVault() }
            </div>
          </div>
          <div className={ styles.tags }>
            <Tags tags={ meta.tags } />
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
        className={ styles.title }
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
    const { isContract } = this.props;
    const { txCount } = this.state;

    if (!txCount || isContract) {
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

  renderVault () {
    const { account } = this.props;
    const { meta } = account;

    if (!meta || !meta.vault) {
      return null;
    }

    return (
      <VaultTag vault={ meta.vault } />
    );
  }
}
