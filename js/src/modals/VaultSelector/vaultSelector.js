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

import Portal from '~/ui/Portal';
import SelectionList from '~/ui/SelectionList';
import VaultCard from '~/ui/VaultCard';

@observer
export default class VaultSelector extends Component {
  static propTypes = {
    onClose: PropTypes.func.isRequired,
    onSelect: PropTypes.func.isRequired,
    selected: PropTypes.string,
    vaultStore: PropTypes.object.isRequired
  };

  render () {
    return (
      <Portal
        isChildModal
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.selector.title'
            defaultMessage='Select Account Vault'
          />
        }
      >
        { this.renderList() }
      </Portal>
    );
  }

  renderList () {
    const { vaultsOpened } = this.props.vaultStore;

    if (vaultsOpened.length === 0) {
      return (
        <FormattedMessage
          id='vaults.selector.noneAvailable'
          defaultMessage='There are currently no vaults opened and available for selection. Create and open some first before attempting to select a vault for an account move.'
        />
      );
    }

    return (
      <SelectionList
        items={ vaultsOpened }
        isChecked={ this.isSelected }
        noStretch
        onSelectClick={ this.onSelect }
        renderItem={ this.renderVault }
      />
    );
  }

  renderVault = (vault) => {
    return (
      <VaultCard
        hideAccounts
        hideButtons
        vault={ vault }
      />
    );
  }

  isSelected = (vault) => {
    return this.props.selected === vault.name;
  }

  onSelect = (vault) => {
    this.props.onSelect(
      this.props.selected === vault.name
        ? ''
        : vault.name
    );
  }

  onClose = () => {
    this.props.onClose();
  }
}
