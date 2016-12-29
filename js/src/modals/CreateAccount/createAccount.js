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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { createIdentityImg } from '~/api/util/identity';
import { newError } from '~/redux/actions';
import { Button, Modal, Warning } from '~/ui';
import { CancelIcon, CheckIcon, DoneIcon, NextIcon, PrevIcon, PrintIcon } from '~/ui/Icons';
import ParityLogo from '~/../assets/images/parity-logo-black-no-text.svg';

import AccountDetails from './AccountDetails';
import AccountDetailsGeth from './AccountDetailsGeth';
import CreationType from './CreationType';
import NewAccount from './NewAccount';
import NewGeth from './NewGeth';
import NewImport from './NewImport';
import RawKey from './RawKey';
import RecoveryPhrase from './RecoveryPhrase';
import Store, { STAGE_CREATE, STAGE_INFO, STAGE_SELECT_TYPE } from './store';
import print from './print';
import recoveryPage from './recoveryPage.ejs';

const TITLES = {
  type:
    <FormattedMessage
      id='createAccount.title.createType'
      defaultMessage='creation type' />,
  create:
    <FormattedMessage
      id='createAccount.title.createAccount'
      defaultMessage='create account' />,
  info:
    <FormattedMessage
      id='createAccount.title.accountInfo'
      defaultMessage='account information' />,
  import:
    <FormattedMessage
      id='createAccount.title.importWallet'
      defaultMessage='import wallet' />
};
const STAGE_NAMES = [TITLES.type, TITLES.create, TITLES.info];
const STAGE_IMPORT = [TITLES.type, TITLES.import, TITLES.info];

@observer
export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    onClose: PropTypes.func,
    onUpdate: PropTypes.func
  }

  store = new Store(this.context.api);

  state = {
    passwordHint: null,
    password: null,
    rawKey: null,
    json: null,
    canCreate: false,
    gethAddresses: []
  }

  render () {
    const { createType, stage } = this.store;

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={
          createType === 'fromNew'
            ? STAGE_NAMES
            : STAGE_IMPORT
        }>
        { this.renderWarning() }
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    const { createType, stage } = this.store;
    const { accounts } = this.props;

    switch (stage) {
      case STAGE_SELECT_TYPE:
        return (
          <CreationType store={ this.store } />
        );

      case STAGE_CREATE:
        if (createType === 'fromNew') {
          return (
            <NewAccount
              onChange={ this.onChangeDetails }
              store={ this.store } />
          );
        } else if (createType === 'fromGeth') {
          return (
            <NewGeth
              accounts={ accounts }
              onChange={ this.onChangeGeth }
              store={ this.store } />
          );
        } else if (createType === 'fromPhrase') {
          return (
            <RecoveryPhrase
              onChange={ this.onChangeDetails }
              store={ this.store } />
          );
        } else if (createType === 'fromRaw') {
          return (
            <RawKey
              onChange={ this.onChangeDetails }
              store={ this.store } />
          );
        }

        return (
          <NewImport
            onChange={ this.onChangeWallet }
            store={ this.store } />
        );

      case STAGE_INFO:
        if (createType === 'fromGeth') {
          return (
            <AccountDetailsGeth
              addresses={ this.state.gethAddresses }
              store={ this.store } />
          );
        }

        return (
          <AccountDetails store={ this.store } />
        );
    }
  }

  renderDialogActions () {
    const { createType, stage } = this.store;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='createAccount.button.cancel'
            defaultMessage='Cancel' />
        }
        onClick={ this.onClose } />
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
                defaultMessage='Next' />
            }
            onClick={ this.store.nextStage } />
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
                defaultMessage='Back' />
            }
            onClick={ this.store.prevStage } />,
          <Button
            disabled={ !this.state.canCreate }
            icon={ <CheckIcon /> }
            key='create'
            label={
              createType === 'fromNew'
                ? <FormattedMessage
                  id='createAccount.button.create'
                  defaultMessage='Create' />
                : <FormattedMessage
                  id='createAccount.button.import'
                  defaultMessage='Import' />
            }
            onClick={ this.onCreate } />
        ];

      case STAGE_INFO:
        return [
          ['fromNew', 'fromPhrase'].includes(createType)
            ? <Button
              icon={ <PrintIcon /> }
              key='print'
              label={
                <FormattedMessage
                  id='createAccount.button.print'
                  defaultMessage='Print Phrase' />
              }
              onClick={ this.printPhrase } />
            : null,
          <Button
            icon={ <DoneIcon /> }
            key='close'
            label={
              <FormattedMessage
                id='createAccount.button.close'
                defaultMessage='Close' />
            }
            onClick={ this.onClose } />
        ];
    }
  }

  renderWarning () {
    const { createType, stage } = this.store;

    if (stage !== STAGE_CREATE || ['fromJSON', 'fromPresale'].includes(createType)) {
      return null;
    }

    return (
      <Warning warning={
        <FormattedMessage
          id='createAccount.warning.insecurePassword'
          defaultMessage='It is recommended that a strong password be used to secure your accounts. Empty and trivial passwords are a security risk.' />
      } />
    );
  }

  onCreate = () => {
    const { createType, isWindowsPhrase, name, phrase } = this.store;
    const { api } = this.context;

    this.setState({
      canCreate: false
    });

    if (['fromNew', 'fromPhrase'].includes(createType)) {
      let formattedPhrase = phrase;
      if (isWindowsPhrase && createType === 'fromPhrase') {
        formattedPhrase = phrase
          .split(' ') // get the words
          .map((word) => word === 'misjudged' ? word : `${word}\r`) // add \r after each (except last in dict)
          .join(' '); // re-create string
      }

      return api.parity
        .newAccountFromPhrase(formattedPhrase, this.state.password)
        .then((address) => {
          this.store.setAddress(address);

          return api.parity
            .setAccountName(address, name)
            .then(() => api.parity.setAccountMeta(address, {
              timestamp: Date.now(),
              passwordHint: this.state.passwordHint
            }));
        })
        .then(() => {
          this.store.nextStage();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          newError(error);
        });
    } else if (createType === 'fromRaw') {
      return api.parity
        .newAccountFromSecret(this.state.rawKey, this.state.password)
        .then((address) => {
          this.store.setAddress(address);

          return api.parity
            .setAccountName(address, name)
            .then(() => api.parity.setAccountMeta(address, {
              timestamp: Date.now(),
              passwordHint: this.state.passwordHint
            }));
        })
        .then(() => {
          this.store.nextStage();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          newError(error);
        });
    } else if (createType === 'fromGeth') {
      return api.parity
        .importGethAccounts(this.state.gethAddresses)
        .then((result) => {
          console.log('result', result);

          return Promise.all(this.state.gethAddresses.map((address) => {
            return api.parity.setAccountName(address, 'Geth Import');
          }));
        })
        .then(() => {
          this.store.nextStage();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          newError(error);
        });
    }

    return api.parity
      .newAccountFromWallet(this.state.json, this.state.password)
      .then((address) => {
        this.store.setAddress(address);

        return api.parity
          .setAccountName(address, name)
          .then(() => api.parity.setAccountMeta(address, {
            timestamp: Date.now(),
            passwordHint: this.state.passwordHint
          }));
      })
      .then(() => {
        this.store.nextStage();
        this.props.onUpdate && this.props.onUpdate();
      })
      .catch((error) => {
        console.error('onCreate', error);

        this.setState({
          canCreate: true
        });

        newError(error);
      });
  }

  onClose = () => {
    this.setState({
      canCreate: false
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  onChangeDetails = (canCreate, { name, passwordHint, address, password, phrase, rawKey }) => {
    this.store.setAddress(address);
    this.store.setName(name);
    this.store.setPhrase(phrase);

    this.setState({
      canCreate,
      password,
      passwordHint,
      rawKey
    });
  }

  onChangeRaw = (canCreate, rawKey) => {
    this.setState({
      canCreate,
      rawKey
    });
  }

  onChangeGeth = (canCreate, gethAddresses) => {
    this.setState({
      canCreate,
      gethAddresses
    });
  }

  onChangeWallet = (canCreate, { name, passwordHint, password, json }) => {
    this.store.setName(name);

    this.setState({
      canCreate,
      json,
      password,
      passwordHint
    });
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
