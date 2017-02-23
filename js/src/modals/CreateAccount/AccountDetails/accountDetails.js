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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { IdentityIcon, Input, QrCode, Title } from '~/ui';

import styles from '../createAccount.css';

@observer
export default class AccountDetails extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { address, description, name } = this.props.store;

    return (
      <div className={ styles.details }>
        <div className={ styles.info }>
          <div className={ styles.account }>
            <div className={ styles.name }>
              <IdentityIcon
                address={ address }
                className={ styles.icon }
                center
              />
              <Title
                byline={ description }
                className={ styles.title }
                title={ name }
              />
            </div>
            <div className={ styles.description }>
              <Input
                readOnly
                hideUnderline
                hint={
                  <FormattedMessage
                    id='createAccount.accountDetails.address.hint'
                    defaultMessage='the network address for the account'
                  />
                }
                label={
                  <FormattedMessage
                    id='createAccount.accountDetails.address.label'
                    defaultMessage='address'
                  />
                }
                value={ address }
                allowCopy={ address }
              />
              { this.renderPhrase() }
            </div>
          </div>
          <QrCode
            className={ styles.qr }
            value={ address }
          />
        </div>
      </div>
    );
  }

  renderPhrase () {
    const { phrase } = this.props.store;

    if (!phrase) {
      return null;
    }

    return (
      <Input
        allowCopy
        hint={
          <FormattedMessage
            id='createAccount.accountDetails.phrase.hint'
            defaultMessage='the account recovery phrase'
          />
        }
        label={
          <FormattedMessage
            id='createAccount.accountDetails.phrase.label'
            defaultMessage='owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)'
          />
        }
        readOnly
        value={ phrase }
      />
    );
  }
}
