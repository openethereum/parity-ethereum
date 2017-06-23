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
    isConfirming: PropTypes.bool,
    withRequiredBackup: PropTypes.bool,
    createStore: PropTypes.object.isRequired
  }

  static defaultPropTypes = {
    isConfirming: false,
    withRequiredBackup: false
  }

  render () {
    const { address, description, name } = this.props.createStore;

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

  renderRequiredBackup () {
    const { phraseBackedUp, phraseBackedUpError } = this.props.createStore;

    if (!this.props.withRequiredBackup) {
      return null;
    }

    return (
      <div>
        <Input
          error={ phraseBackedUpError }
          hint={
            <FormattedMessage
              id='createAccount.accountDetails.phrase.hint'
              defaultMessage='the account recovery phrase'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.accountDetails.phrase.backedUp'
              defaultMessage='Type "I have written down the phrase" below to confirm it is backed up.'
            />
          }
          onChange={ this.onEditPhraseBackedUp }
          value={ phraseBackedUp }
        />
      </div>
    );
  }

  renderPhrase () {
    const { isConfirming } = this.props;
    const { isTest, phrase, backupPhraseError } = this.props.createStore;

    const hint = (
      <FormattedMessage
        id='createAccount.accountDetails.phrase.hint'
        defaultMessage='the account recovery phrase'
      />
    );
    const label = (
      <FormattedMessage
        id='createAccount.accountDetails.phrase.label'
        defaultMessage='owner recovery phrase'
      />
    );

    if (!isConfirming) {
      if (!phrase) {
        return null;
      }

      return (
        <div>
          <Input
            allowCopy
            hint={ hint }
            label={ label }
            readOnly
            value={ phrase }
          />
          <div className={ styles.backupPhrase }>
            <FormattedMessage
              id='createAccount.accountDetails.phrase.backup'
              defaultMessage='Please back up the recovery phrase now. Make sure to keep it private and secure, it allows full and unlimited access to the account.'
            />
          </div>
          { this.renderRequiredBackup() }
        </div>
      );
    }

    return (
      <div>
        <Input
          allowPaste={ isTest }
          error={ backupPhraseError }
          hint={ hint }
          label={ label }
          onChange={ this.onEditPhrase }
          value={ phrase }
        />
        <div className={ styles.backupPhrase }>
          <FormattedMessage
            id='createAccount.accountDetails.phrase.backupConfirm'
            defaultMessage='Type your recovery phrase now.'
          />
        </div>
      </div>
    );
  }

  onEditPhraseBackedUp = (ev) => {
    this.props.createStore.setPhraseBackedUp(ev.target.value);
  }

  onEditPhrase = (ev) => {
    this.props.createStore.setPhrase(ev.target.value);
  }
}
