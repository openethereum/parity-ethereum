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
        id='createAccount.creationType.fromQr.description'
        defaultMessage='Attach an externally managed account via QR code'
      />
    ),
    label: (
      <FormattedMessage
        id='createAccount.creationType.fromQr.label'
        defaultMessage='External Account'
      />
    ),
    key: 'fromQr'
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
  }
];

@observer
export default class CreationType extends Component {
  static propTypes = {
    createStore: PropTypes.object.isRequired
  }

  render () {
    const { createType } = this.props.createStore;

    return (
      <div>
        <div className={ styles.summary }>
          <FormattedMessage
            id='createAccount.creationType.info'
            defaultMessage='Please select the type of account you want to create. Either create an account via name & password, or import it from a variety of existing sources. From here the wizard will guide you through the process of completing your account creation.'
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
        onSelectDoubleClick={ this.onSelect }
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
            createStore={ this.props.createStore }
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
    const { createType } = this.props.createStore;

    return item.key === createType;
  }

  onChange = (item) => {
    const { createStore } = this.props;

    createStore.setCreateType(item.key);
  }

  onSelect = (item) => {
    const { createStore } = this.props;

    createStore.setCreateType(item.key);
    createStore.nextStage();
  }
}
