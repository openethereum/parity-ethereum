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
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import styles from '../createAccount.css';

@observer
export default class CreationType extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { createType } = this.props.store;

    return (
      <div className={ styles.spaced }>
        <RadioButtonGroup
          defaultSelected={ createType }
          name='creationType'
          onChange={ this.onChange }
        >
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromNew.label'
                defaultMessage='Create new account manually'
              />
            }
            value='fromNew'
          />
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromPhrase.label'
                defaultMessage='Recover account from recovery phrase'
              />
            }
            value='fromPhrase'
          />
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromGeth.label'
                defaultMessage='Import accounts from Geth keystore'
              />
            }
            value='fromGeth'
          />
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromJSON.label'
                defaultMessage='Import account from a backup JSON file'
              />
            }
            value='fromJSON'
          />
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromPresale.label'
                defaultMessage='Import account from an Ethereum pre-sale wallet'
              />
            }
            value='fromPresale'
          />
          <RadioButton
            label={
              <FormattedMessage
                id='createAccount.creationType.fromRaw.label'
                defaultMessage='Import raw private key'
              />
            }
            value='fromRaw'
          />
        </RadioButtonGroup>
      </div>
    );
  }

  onChange = (event) => {
    const { store } = this.props;

    store.setCreateType(event.target.value);
  }
}
