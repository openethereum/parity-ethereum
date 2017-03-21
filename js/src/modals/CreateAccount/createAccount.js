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

import ParityLogo from '~/../assets/images/parity-logo-black-no-text.svg';
import { createIdentityImg } from '~/api/util/identity';
import { newError } from '~/redux/actions';
import { Button, ModalBox, Portal } from '~/ui';
import { CancelIcon, CheckIcon, DoneIcon, NextIcon, PrevIcon, PrintIcon } from '~/ui/Icons';

import VaultStore from '~/views/Vaults/store';

import AccountDetails from './AccountDetails';
import AccountDetailsGeth from './AccountDetailsGeth';
import CreationType from './CreationType';
import NewAccount from './NewAccount';
import NewGeth from './NewGeth';
import NewImport from './NewImport';
import RawKey from './RawKey';
import RecoveryPhrase from './RecoveryPhrase';
import Store, { STAGE_CREATE, STAGE_INFO, STAGE_SELECT_TYPE } from './store';
import TypeIcon from './TypeIcon';
import print from './print';
import recoveryPage from './recoveryPage.ejs';

const TITLES = {
  type: (
    <FormattedMessage
      id='createAccount.title.createType'
      defaultMessage='creation type'
    />
  ),
  create: (
    <FormattedMessage
      id='createAccount.title.createAccount'
      defaultMessage='create account'
    />
  ),
  info: (
    <FormattedMessage
      id='createAccount.title.accountInfo'
      defaultMessage='account information'
    />
  ),
  import: (
    <FormattedMessage
      id='createAccount.title.importWallet'
      defaultMessage='import wallet'
    />
  )
};
const STAGE_NAMES = [TITLES.type, TITLES.create, TITLES.info];
const STAGE_IMPORT = [TITLES.type, TITLES.import, TITLES.info];

@observer
class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func,
    onUpdate: PropTypes.func
  }

  store = new Store(this.context.api, this.props.accounts);
  vaultStore = VaultStore.get(this.context.api);

  componentWillMount () {
    return this.vaultStore.loadVaults();
  }

  render () {
    const { isBusy, createType, stage } = this.store;

    return (
      <Portal
        buttons={ this.renderDialogActions() }
        busy={ isBusy }
        activeStep={ stage }
        onClose={ this.onClose }
        open
        steps={
          createType === 'fromNew'
            ? STAGE_NAMES
            : STAGE_IMPORT
        }
      >
        <ModalBox icon={ <TypeIcon store={ this.store } /> }>
          { this.renderPage() }
        </ModalBox>
      </Portal>
    );
  }

  renderPage () {
    const { createType, stage } = this.store;

    switch (stage) {
      case STAGE_SELECT_TYPE:
        return (
          <CreationType store={ this.store } />
        );

      case STAGE_CREATE:
        if (createType === 'fromNew') {
          return (
            <NewAccount
              newError={ this.props.newError }
              store={ this.store }
              vaultStore={ this.vaultStore }
            />
          );
        }

        if (createType === 'fromGeth') {
          return (
            <NewGeth store={ this.store } />
          );
        }

        if (createType === 'fromPhrase') {
          return (
            <RecoveryPhrase
              store={ this.store }
              vaultStore={ this.vaultStore }
            />
          );
        }

        if (createType === 'fromRaw') {
          return (
            <RawKey
              store={ this.store }
              vaultStore={ this.vaultStore }
            />
          );
        }

        return (
          <NewImport
            store={ this.store }
            vaultStore={ this.vaultStore }
          />
        );

      case STAGE_INFO:
        if (createType === 'fromGeth') {
          return (
            <AccountDetailsGeth store={ this.store } />
          );
        }

        return (
          <AccountDetails store={ this.store } />
        );
    }
  }

  renderDialogActions () {
    const { createType, canCreate, isBusy, stage } = this.store;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='createAccount.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />
    );

    switch (stage) {
      case STAGE_SELECT_TYPE:
        return [
          cancelBtn,
          <Button
            icon={ <NextIcon /> }
            key='next'
            label={
              <FormattedMessage
                id='createAccount.button.next'
                defaultMessage='Next'
              />
            }
            onClick={ this.store.nextStage }
          />
        ];

      case STAGE_CREATE:
        return [
          cancelBtn,
          <Button
            icon={ <PrevIcon /> }
            key='back'
            label={
              <FormattedMessage
                id='createAccount.button.back'
                defaultMessage='Back'
              />
            }
            onClick={ this.store.prevStage }
          />,
          <Button
            disabled={ !canCreate || isBusy }
            icon={ <CheckIcon /> }
            key='create'
            label={
              createType === 'fromNew'
                ? (
                  <FormattedMessage
                    id='createAccount.button.create'
                    defaultMessage='Create'
                  />
                )
                : (
                  <FormattedMessage
                    id='createAccount.button.import'
                    defaultMessage='Import'
                  />
                )
            }
            onClick={ this.onCreate }
          />
        ];

      case STAGE_INFO:
        return [
          ['fromNew', 'fromPhrase'].includes(createType)
            ? (
              <Button
                icon={ <PrintIcon /> }
                key='print'
                label={
                  <FormattedMessage
                    id='createAccount.button.print'
                    defaultMessage='Print Phrase'
                  />
                }
                onClick={ this.printPhrase }
              />
            )
            : null,
          <Button
            icon={ <DoneIcon /> }
            key='done'
            label={
              <FormattedMessage
                id='createAccount.button.done'
                defaultMessage='Done'
              />
            }
            onClick={ this.onClose }
          />
        ];
    }
  }

  onCreate = () => {
    this.store.setBusy(true);

    return this.store
      .createAccount(this.vaultStore)
      .then(() => {
        this.store.setBusy(false);
        this.store.nextStage();
        this.props.onUpdate && this.props.onUpdate();
      })
      .catch((error) => {
        this.store.setBusy(false);
        this.props.newError(error);
      });
  }

  onClose = () => {
    this.props.onClose && this.props.onClose();
  }

  printPhrase = () => {
    const { address, name, phrase } = this.store;
    const identity = createIdentityImg(address);

    print(recoveryPage({
      address,
      identity,
      logo: ParityLogo,
      name,
      phrase
    }));
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
)(CreateAccount);
