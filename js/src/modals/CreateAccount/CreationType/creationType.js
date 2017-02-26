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

import { Container, SelectionList, Title } from '~/ui';

import TypeIcon from '../TypeIcon';
import styles from '../createAccount.css';

const TYPES = [
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromNew.description'
        defaultMessage='Selecting your identity icon and specifying the password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromNew.label'
        defaultMessage='New Account'
      />
    ),
    key: 'fromNew'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromPhrase.description'
        defaultMessage='Recover using a previously stored recovery phrase and new password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromPhrase.label'
        defaultMessage='Recovery phrase'
      />
    ),
    key: 'fromPhrase'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromGeth.description'
        defaultMessage='Import accounts from the Geth keystore with the original password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromGeth.label'
        defaultMessage='Geth keystore'
      />
    ),
    key: 'fromGeth'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromJSON.description'
        defaultMessage='Import an industry-standard JSON keyfile with the original password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromJSON.label'
        defaultMessage='JSON file'
      />
    ),
    key: 'fromJSON'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromPresale.description'
        defaultMessage='Import an Ethereum presale wallet file with the original password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromPresale.label'
        defaultMessage='Presale wallet'
      />
    ),
    key: 'fromPresale'
  },
  {
    description: (
      <FormattedMessage
        id='createAccount.creationType.fromRaw.description'
        defaultMessage='Enter a previously created raw private key with a new password'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromRaw.label'
        defaultMessage='Private key'
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
      <div>
        <div className={ styles.summary }>
          <FormattedMessage
            id='createAccount.creationType.info'
            defaultMessage='Please select the type of account you want to create. Either create an account via name & password, or import it from a variety of existing sources. From here the wizard will guid you through the process of completing your account creation.'
          />
        </div>
        { this.renderList(createType) }
      </div>
    );
  }

  renderList () {
    return (
      <SelectionList
        isChecked={ this.isSelected }
        items={ TYPES }
        noStretch
        onSelectClick={ this.onChange }
        renderItem={ this.renderItem }
      />
    );
  }

  renderItem = (item) => {
    return (
      <Container>
        <div className={ styles.selectItem }>
          <TypeIcon
            className={ styles.icon }
            store={ this.props.store }
            type={ item.key }
          />
          <Title
            byline={ item.description }
            className={ styles.info }
            title={ item.label }
          />
        </div>
      </Container>
    );
  }

  isSelected = (item) => {
    const { createType } = this.props.store;

    return item.key === createType;
  }

  onChange = (item) => {
    const { store } = this.props;

    store.setCreateType(item.key);
  }
}
