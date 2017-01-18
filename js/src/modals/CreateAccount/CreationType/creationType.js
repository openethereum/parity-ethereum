// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import styles from '../createAccount.css';

export default class CreationType extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  componentWillMount () {
    this.props.onChange('fromNew');
  }

  render () {
    return (
      <div className={ styles.spaced }>
        <RadioButtonGroup
          defaultSelected='fromNew'
          name='creationType'
          onChange={ this.onChange }
        >
          <RadioButton
            label='Create new account manually'
            value='fromNew'
          />
          <RadioButton
            label='Recover account from recovery phrase'
            value='fromPhrase'
          />
          <RadioButton
            label='Import accounts from Geth keystore'
            value='fromGeth'
          />
          <RadioButton
            label='Import account from a backup JSON file'
            value='fromJSON'
          />
          <RadioButton
            label='Import account from an Ethereum pre-sale wallet'
            value='fromPresale'
          />
          <RadioButton
            label='Import raw private key'
            value='fromRaw'
          />
        </RadioButtonGroup>
      </div>
    );
  }

  onChange = (event) => {
    this.props.onChange(event.target.value);
  }
}
