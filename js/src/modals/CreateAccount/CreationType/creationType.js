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

import { RadioButtons } from '~/ui';

import styles from '../createAccount.css';

const TYPES = [
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromNew.description'
        defaultMessage='Create an account by selecting your identity icon and specifying the password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromNew.label'
        defaultMessage='Create new account manually'
      />
    ),
    key: 'fromNew'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromPhrase.description'
        defaultMessage='Recover an account by entering a previously stored recovery phrase and new password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromPhrase.label'
        defaultMessage='Recover account from recovery phrase'
      />
    ),
    key: 'fromPhrase'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromGeth.description'
        defaultMessage='Import an accounts from the Geth keystore with the original password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromGeth.label'
        defaultMessage='Import accounts from Geth keystore'
      />
    ),
    key: 'fromGeth'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromJSON.description'
        defaultMessage='Create an account by importing an industry-standard JSON keyfile'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromJSON.label'
        defaultMessage='Import account from a backup JSON file'
      />
    ),
    key: 'fromJSON'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromPresale.description'
        defaultMessage='Create an account by importing an Ethereum presale wallet file'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromPresale.label'
        defaultMessage='Import account from an Ethereum pre-sale wallet'
      />
    ),
    key: 'fromPresale'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromRaw.description'
        defaultMessage='Create an account by entering a previously backed-up raw private key'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromRaw.label'
        defaultMessage='Import raw private key'
      />
    ),
    key: 'fromRaw'
  }
];

@observer
export default class CreationType extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { createType } = this.props.store;

    return (
      <div className={ styles.spaced }>
        <RadioButtons
          name='creationType'
          onChange={ this.onChange }
          value={ createType }
          values={ TYPES }
        />
      </div>
    );
  }

  onChange = (event) => {
    const { store } = this.props;

    store.setCreateType(event.target.value);
  }
}
