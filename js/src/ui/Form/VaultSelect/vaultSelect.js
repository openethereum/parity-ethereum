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

import VaultSelector from '~/modals/VaultSelector';
import VaultStore from '~/views/Vaults/store';

import InputAddress from '../InputAddress';

export default class VaultSelect extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    onSelect: PropTypes.func.isRequired,
    value: PropTypes.string,
    vaultStore: PropTypes.object
  };

  state = {
    isOpen: false
  };

  vaultStore = this.props.vaultStore || VaultStore.get(this.context.api);

  componentWillMount () {
    return this.vaultStore.loadVaults();
  }

  render () {
    const { value } = this.props;

    return (
      <div>
        { this.renderSelector() }
        <InputAddress
          allowCopy={ false }
          allowInvalid
          disabled
          hint={
            <FormattedMessage
              id='ui.vaultSelect.hint'
              defaultMessage='the vault this account is attached to'
            />
          }
          label={
            <FormattedMessage
              id='ui.vaultSelect.label'
              defaultMessage='associated vault'
            />
          }
          onClick={ this.openSelector }
          value={ (value || '').toUpperCase() }
        />
      </div>
    );
  }

  renderSelector () {
    const { value } = this.props;
    const { isOpen } = this.state;

    if (!isOpen) {
      return null;
    }

    return (
      <VaultSelector
        onClose={ this.closeSelector }
        onSelect={ this.onSelect }
        selected={ value }
        vaultStore={ this.vaultStore }
      />
    );
  }

  openSelector = () => {
    this.setState({
      isOpen: true
    });
  }

  closeSelector = () => {
    this.setState({
      isOpen: false
    });
  }

  onSelect = (vaultName) => {
    this.props.onSelect(vaultName);
    this.closeSelector();
  }
}
