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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError } from '@parity/shared/redux/actions';
import { Button, Form, Input, InputChip, Portal, VaultSelect } from '@parity/ui';
import { CancelIcon, SaveIcon } from '@parity/ui/Icons';

import VaultStore from '@parity/dapp-vaults/store';

import Store from './store';

@observer
class EditMeta extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  }

  store = new Store(this.context.api, this.props.account);
  vaultStore = VaultStore.get(this.context.api);

  componentWillMount () {
    this.vaultStore.loadVaults();
  }

  render () {
    const { description, isBusy, name, nameError, tags } = this.store;

    return (
      <Portal
        buttons={ this.renderActions() }
        busy={ isBusy }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='editMeta.title'
            defaultMessage='edit metadata'
          />
        }
      >
        <Form>
          <Input
            autoFocus
            error={ nameError }
            label={
              <FormattedMessage
                id='editMeta.name.label'
                defaultMessage='name'
              />
            }
            onSubmit={ this.store.setName }
            value={ name }
          />
          <Input
            hint={
              <FormattedMessage
                id='editMeta.description.hint'
                defaultMessage='description for this address'
              />
            }
            label={
              <FormattedMessage
                id='editMeta.description.label'
                defaultMessage='address description'
              />
            }
            value={ description }
            onSubmit={ this.store.setDescription }
          />
          { this.renderAccountFields() }
          <InputChip
            addOnBlur
            hint={
              <FormattedMessage
                id='editMeta.tags.hint'
                defaultMessage='press <Enter> to add a tag'
              />
            }
            label={
              <FormattedMessage
                id='editMeta.tags.label'
                defaultMessage='(optional) tags'
              />
            }
            onTokensChange={ this.store.setTags }
            tokens={ tags.slice() }
          />
          { this.renderVaultSelector() }
        </Form>
      </Portal>
    );
  }

  renderActions () {
    const { hasError } = this.store;

    return [
      <Button
        label='Cancel'
        icon={ <CancelIcon /> }
        key='cancel'
        onClick={ this.onClose }
      />,
      <Button
        disabled={ hasError }
        label='Save'
        icon={ <SaveIcon /> }
        key='save'
        onClick={ this.onSave }
      />
    ];
  }

  renderAccountFields () {
    const { isAccount, passwordHint } = this.store;

    if (!isAccount) {
      return null;
    }

    return (
      <Input
        hint={
          <FormattedMessage
            id='editMeta.passwordHint.hint'
            defaultMessage='a hint to allow password recovery'
          />
        }
        label={
          <FormattedMessage
            id='editMeta.passwordHint.label'
            defaultMessage='(optional) password hint'
          />
        }
        value={ passwordHint }
        onSubmit={ this.store.setPasswordHint }
      />
    );
  }

  renderVaultSelector () {
    const { isAccount, vaultName } = this.store;

    if (!isAccount) {
      return null;
    }

    return (
      <VaultSelect
        onSelect={ this.setVaultName }
        value={ vaultName }
        vaultStore={ this.vaultStore }
      />
    );
  }

  onClose = () => {
    this.props.onClose();
  }

  onSave = () => {
    if (this.store.hasError) {
      return;
    }

    return this.store
      .save(this.vaultStore)
      .then(this.onClose)
      .catch((error) => {
        this.props.newError(error);
      });
  }

  setVaultName = (vaultName) => {
    this.store.setVaultName(vaultName);
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(EditMeta);
