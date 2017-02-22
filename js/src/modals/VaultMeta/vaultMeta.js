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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError } from '~/redux/actions';
import { Button, Input, InputChip, Form, Portal, VaultCard } from '~/ui';
import { CheckIcon, CloseIcon } from '~/ui/Icons';

@observer
class VaultMeta extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  };

  render () {
    const { isBusyMeta, isModalMetaOpen, vault, vaultDescription, vaultTags } = this.props.vaultStore;

    if (!isModalMetaOpen) {
      return null;
    }

    return (
      <Portal
        busy={ isBusyMeta }
        buttons={ [
          <Button
            disabled={ isBusyMeta }
            icon={ <CloseIcon /> }
            key='close'
            label={
              <FormattedMessage
                id='vaults.editMeta.button.close'
                defaultMessage='close'
              />
            }
            onClick={ this.onClose }
          />,
          <Button
            disabled={ isBusyMeta }
            icon={ <CheckIcon /> }
            key='vault'
            label={
              <FormattedMessage
                id='vaults.editMeta.button.save'
                defaultMessage='save'
              />
            }
            onClick={ this.onCreate }
          />
        ] }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.editMeta.title'
            defaultMessage='Edit Vault Metadata'
          />
        }
      >
        <VaultCard.Layout
          withBorder
          vault={ vault }
        />
        <Form>
          <Input
            hint={
              <FormattedMessage
                id='vaults.editMeta.description.hint'
                defaultMessage='the description for this vault'
              />
            }
            label={
              <FormattedMessage
                id='vaults.editMeta.description.label'
                defaultMessage='vault description'
              />
            }
            onChange={ this.onChangeDescription }
            value={ vaultDescription }
          />
          <InputChip
            addOnBlur
            hint={
              <FormattedMessage
                id='vaults.editMeta.tags.hint'
                defaultMessage='press <Enter> to add a tag'
              />
            }
            label={
              <FormattedMessage
                id='vaults.editMeta.tags.label'
                defaultMessage='(optional) tags'
              />
            }
            onTokensChange={ this.onChangeTags }
            tokens={ vaultTags.slice() }
          />
        </Form>
      </Portal>
    );
  }

  onChangeDescription = (event, description) => {
    this.props.vaultStore.setVaultDescription(description);
  }

  onChangeTags = (tags) => {
    this.props.vaultStore.setVaultTags(tags);
  }

  onExecute = () => {
    return this.props.vaultStore
      .editVault()
      .catch(this.props.newError)
      .then(this.onClose);
  }

  onClose = () => {
    this.props.vaultStore.closeMetaModal();
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
)(VaultMeta);
