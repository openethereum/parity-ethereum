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
import NewQr from './NewQr';
import RawKey from './RawKey';
import RecoveryPhrase from './RecoveryPhrase';
import Store, { STAGE_CREATE, STAGE_INFO, STAGE_SELECT_TYPE, STAGE_CONFIRM_BACKUP } from './store';
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
  backup: (
    <FormattedMessage
      id='createAccount.title.backupPhrase'
      defaultMessage='confirm recovery phrase'
    />
  ),
  import: (
    <FormattedMessage
      id='createAccount.title.importAccount'
      defaultMessage='import account'
    />
  ),
  restore: (
    <FormattedMessage
      id='createAccount.title.restoreAccount'
      defaultMessage='restore account'
    />
  ),
  qr: (
    <FormattedMessage
      id='createAccount.title.qr'
      defaultMessage='external account'
    />
  )
};
const STAGE_NAMES = [TITLES.type, TITLES.create, TITLES.info, TITLES.backup];
const STAGE_IMPORT = [TITLES.type, TITLES.import, TITLES.info];
const STAGE_RESTORE = [TITLES.restore, TITLES.info];
const STAGE_QR = [TITLES.type, TITLES.qr, TITLES.info];

@observer
class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    isTest: PropTypes.bool.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func,
    onUpdate: PropTypes.func,
    restore: PropTypes.bool
  };

  static defaultProps = {
    restore: false
  };

  createStore = new Store(this.context.api, this.props.accounts, this.props.isTest);
  vaultStore = VaultStore.get(this.context.api);

  componentWillMount () {
    if (this.props.restore) {
      this.createStore.setCreateType('fromPhrase');
      this.createStore.nextStage();
    }

    return this.vaultStore.loadVaults();
  }

  render () {
    const { isBusy, createType, stage } = this.createStore;

    let steps = STAGE_IMPORT;

    if (createType === 'fromNew') {
      steps = STAGE_NAMES;
    } else if (createType === 'fromQr') {
      steps = STAGE_QR;
    } else if (createType === 'fromPhrase') {
      steps = STAGE_RESTORE;
    }

    return (
      <Portal
        buttons={ this.renderDialogActions() }
        busy={ isBusy }
        activeStep={ stage }
        onClose={ this.onClose }
        open
        steps={ steps }
      >
        <ModalBox icon={ <TypeIcon createStore={ this.createStore } /> }>
          { this.renderPage() }
        </ModalBox>
      </Portal>
    );
  }

  renderPage () {
    const { createType, stage } = this.createStore;

    switch (stage) {
      case STAGE_SELECT_TYPE:
        return (
          <CreationType createStore={ this.createStore } />
        );

      case STAGE_CREATE:
        if (createType === 'fromNew') {
          return (
            <NewAccount
              newError={ this.props.newError }
              createStore={ this.createStore }
              vaultStore={ this.vaultStore }
            />
          );
        }

        if (createType === 'fromGeth') {
          return (
            <NewGeth createStore={ this.createStore } />
          );
        }

        if (createType === 'fromPhrase') {
          return (
            <RecoveryPhrase
              createStore={ this.createStore }
              vaultStore={ this.vaultStore }
            />
          );
        }

        if (createType === 'fromQr') {
          return (
            <NewQr
              createStore={ this.createStore }
              vaultStore={ this.vaultStore }
            />
          );
        }

        if (createType === 'fromRaw') {
          return (
            <RawKey
              createStore={ this.createStore }
              vaultStore={ this.vaultStore }
            />
          );
        }

        return (
          <NewImport
            createStore={ this.createStore }
            vaultStore={ this.vaultStore }
          />
        );

      case STAGE_INFO:
        if (createType === 'fromGeth') {
          return (
            <AccountDetailsGeth createStore={ this.createStore } />
          );
        }

        return (
          <AccountDetails
            createStore={ this.createStore }
            withRequiredBackup={ createType === 'fromNew' }
          />
        );

      case STAGE_CONFIRM_BACKUP:
        return (
          <AccountDetails
            createStore={ this.createStore }
            isConfirming
          />
        );
    }
  }

  renderDialogActions () {
    const { restore } = this.props;
    const { createType, canCreate, isBusy, stage, phraseBackedUpError } = this.createStore;

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

    const backBtn = restore
      ? null
      : (
        <Button
          icon={ <PrevIcon /> }
          key='back'
          label={
            <FormattedMessage
              id='createAccount.button.back'
              defaultMessage='Back'
            />
          }
          onClick={ this.createStore.prevStage }
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
            onClick={ this.createStore.nextStage }
          />
        ];

      case STAGE_CREATE:
        return [
          cancelBtn,
          backBtn,
          <Button
            disabled={ !canCreate || isBusy }
            icon={ <CheckIcon /> }
            key='create'
            label={
              createType === 'fromNew'
                ? (
                  <FormattedMessage
                    id='createAccount.button.next'
                    defaultMessage='Next'
                  />
                )
                : (
                  <FormattedMessage
                    id='createAccount.button.import'
                    defaultMessage='Import'
                  />
                )
            }
            onClick={ createType === 'fromNew' ? this.createStore.nextStage : this.onCreate }
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
            disabled={ createType === 'fromNew' && !!phraseBackedUpError }
            icon={ <DoneIcon /> }
            key='done'
            label={
              <FormattedMessage
                id='createAccount.button.done'
                defaultMessage='Done'
              />
            }
            onClick={ createType === 'fromNew' ? this.onConfirmPhraseBackup : this.onClose }
          />
        ];

      case STAGE_CONFIRM_BACKUP:
        return [
          <Button
            icon={ <DoneIcon /> }
            key='done'
            label={
              <FormattedMessage
                id='createAccount.button.create'
                defaultMessage='Create'
              />
            }
            onClick={ this.onCreateNew }
          />
        ];
    }
  }

  onConfirmPhraseBackup = () => {
    this.createStore.clearPhrase();
    this.createStore.nextStage();
  }

  onCreateNew = () => {
    this.createStore.setBusy(true);
    this.createStore.computeBackupPhraseAddress()
      .then(err => {
        if (err) {
          this.createStore.setBusy(false);
          return;
        }

        return this.createStore.createAccount(this.vaultStore)
          .then(() => {
            this.createStore.clearPhrase();
            this.createStore.setBusy(false);
            this.props.onUpdate && this.props.onUpdate();
            this.onClose();
          });
      })
      .catch((error) => {
        this.createStore.setBusy(false);
        this.props.newError(error);
      });
  }

  onCreate = () => {
    return this.createStore
      .createAccount(this.vaultStore)
      .then(() => {
        this.createStore.nextStage();
        this.props.onUpdate && this.props.onUpdate();
      })
      .catch((error) => {
        this.props.newError(error);
      });
  }

  onClose = () => {
    this.createStore.clearPhrase();
    this.props.onClose && this.props.onClose();
  }

  printPhrase = () => {
    const { address, name, phrase } = this.createStore;
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

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;

  return { isTest };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(CreateAccount);
