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
import { FormattedMessage } from 'react-intl';

import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';
import PrintIcon from 'material-ui/svg-icons/action/print';

import { Button, Modal, Warning } from '~/ui';

import AccountDetails from './AccountDetails';
import AccountDetailsGeth from './AccountDetailsGeth';
import CreationType from './CreationType';
import NewAccount from './NewAccount';
import NewGeth from './NewGeth';
import NewImport from './NewImport';
import RawKey from './RawKey';
import RecoveryPhrase from './RecoveryPhrase';

import { createIdentityImg } from '~/api/util/identity';
import print from './print';
import recoveryPage from './recovery-page.ejs';
import ParityLogo from '../../../assets/images/parity-logo-black-no-text.svg';

const TITLES = {
  type: 'creation type',
  create: 'create account',
  info: 'account information',
  import: 'import wallet'
};
const STAGE_NAMES = [TITLES.type, TITLES.create, TITLES.info];
const STAGE_IMPORT = [TITLES.type, TITLES.import, TITLES.info];

export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    onClose: PropTypes.func,
    onUpdate: PropTypes.func
  }

  state = {
    address: null,
    name: null,
    passwordHint: null,
    password: null,
    phrase: null,
    windowsPhrase: false,
    rawKey: null,
    json: null,
    canCreate: false,
    createType: null,
    gethAddresses: [],
    stage: 0
  }

  render () {
    const { createType, stage } = this.state;
    const steps = createType === 'fromNew'
      ? STAGE_NAMES
      : STAGE_IMPORT;

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ steps }
      >
        { this.renderWarning() }
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    const { createType, stage } = this.state;
    const { accounts } = this.props;

    switch (stage) {
      case 0:
        return (
          <CreationType onChange={ this.onChangeType } />
        );

      case 1:
        if (createType === 'fromNew') {
          return (
            <NewAccount onChange={ this.onChangeDetails } />
          );
        }

        if (createType === 'fromGeth') {
          return (
            <NewGeth
              accounts={ accounts }
              onChange={ this.onChangeGeth }
            />
          );
        }

        if (createType === 'fromPhrase') {
          return (
            <RecoveryPhrase onChange={ this.onChangeDetails } />
          );
        }

        if (createType === 'fromRaw') {
          return (
            <RawKey onChange={ this.onChangeDetails } />
          );
        }

        return (
          <NewImport onChange={ this.onChangeWallet } />
        );

      case 2:
        if (createType === 'fromGeth') {
          return (
            <AccountDetailsGeth addresses={ this.state.gethAddresses } />
          );
        }

        return (
          <AccountDetails
            address={ this.state.address }
            name={ this.state.name }
            phrase={ this.state.phrase }
          />
        );
    }
  }

  renderDialogActions () {
    const { createType, stage } = this.state;

    switch (stage) {
      case 0:
        return [
          <Button
            icon={ <ContentClear /> }
            label='Cancel'
            onClick={ this.onClose }
          />,
          <Button
            icon={ <NavigationArrowForward /> }
            label='Next'
            onClick={ this.onNext }
          />
        ];
      case 1:
        const createLabel = createType === 'fromNew'
          ? 'Create'
          : 'Import';

        return [
          <Button
            icon={ <ContentClear /> }
            label='Cancel'
            onClick={ this.onClose }
          />,
          <Button
            icon={ <NavigationArrowBack /> }
            label='Back'
            onClick={ this.onPrev }
          />,
          <Button
            icon={ <ActionDone /> }
            label={ createLabel }
            disabled={ !this.state.canCreate }
            onClick={ this.onCreate }
          />
        ];

      case 2:
        return [
          createType === 'fromNew' || createType === 'fromPhrase' ? (
            <Button
              icon={ <PrintIcon /> }
              label='Print Phrase'
              onClick={ this.printPhrase }
            />
          ) : null,
          <Button
            icon={ <ActionDoneAll /> }
            label='Close'
            onClick={ this.onClose }
          />
        ];
    }
  }

  renderWarning () {
    const { createType, stage } = this.state;

    if (stage !== 1 || ['fromJSON', 'fromPresale'].includes(createType)) {
      return null;
    }

    return (
      <Warning
        warning={
          <FormattedMessage
            id='createAccount.warning.insecurePassword'
            defaultMessage='It is recommended that a strong password be used to secure your accounts. Empty and trivial passwords are a security risk.'
          />
        }
      />
    );
  }

  onNext = () => {
    this.setState({
      stage: this.state.stage + 1
    });
  }

  onPrev = () => {
    this.setState({
      stage: this.state.stage - 1
    });
  }

  onCreate = () => {
    const { createType, windowsPhrase } = this.state;
    const { api } = this.context;

    this.setState({
      canCreate: false
    });

    if (createType === 'fromNew' || createType === 'fromPhrase') {
      let phrase = this.state.phrase;

      if (createType === 'fromPhrase' && windowsPhrase) {
        phrase = phrase
          .split(' ') // get the words
          .map((word) => word === 'misjudged' ? word : `${word}\r`) // add \r after each (except last in dict)
          .join(' '); // re-create string
      }

      return api.parity
        .newAccountFromPhrase(phrase, this.state.password)
        .then((address) => {
          this.setState({ address });
          return api.parity
            .setAccountName(address, this.state.name)
            .then(() => api.parity.setAccountMeta(address, {
              timestamp: Date.now(),
              passwordHint: this.state.passwordHint
            }));
        })
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          this.newError(error);
        });
    }

    if (createType === 'fromRaw') {
      return api.parity
        .newAccountFromSecret(this.state.rawKey, this.state.password)
        .then((address) => {
          this.setState({ address });
          return api.parity
            .setAccountName(address, this.state.name)
            .then(() => api.parity.setAccountMeta(address, {
              timestamp: Date.now(),
              passwordHint: this.state.passwordHint
            }));
        })
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          this.newError(error);
        });
    }

    if (createType === 'fromGeth') {
      return api.parity
        .importGethAccounts(this.state.gethAddresses)
        .then((result) => {
          console.log('result', result);

          return Promise.all(this.state.gethAddresses.map((address) => {
            return api.parity.setAccountName(address, 'Geth Import');
          }));
        })
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          console.error('onCreate', error);

          this.setState({
            canCreate: true
          });

          this.newError(error);
        });
    }

    return api.parity
      .newAccountFromWallet(this.state.json, this.state.password)
      .then((address) => {
        this.setState({
          address: address
        });

        return api.parity
          .setAccountName(address, this.state.name)
          .then(() => api.parity.setAccountMeta(address, {
            timestamp: Date.now(),
            passwordHint: this.state.passwordHint
          }));
      })
      .then(() => {
        this.onNext();
        this.props.onUpdate && this.props.onUpdate();
      })
      .catch((error) => {
        console.error('onCreate', error);

        this.setState({
          canCreate: true
        });

        this.newError(error);
      });
  }

  onClose = () => {
    this.setState({
      stage: 0,
      canCreate: false
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  onChangeType = (value) => {
    this.setState({
      createType: value
    });
  }

  onChangeDetails = (canCreate, { name, passwordHint, address, password, phrase, rawKey, windowsPhrase }) => {
    const nextState = {
      canCreate,
      name,
      passwordHint,
      address,
      password,
      phrase,
      windowsPhrase: windowsPhrase || false,
      rawKey
    };

    this.setState(nextState);
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
    this.setState({
      canCreate,
      name,
      passwordHint,
      password,
      json
    });
  }

  newError = (error) => {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }

  printPhrase = () => {
    const { address, phrase, name } = this.state;
    const identity = createIdentityImg(address);

    print(recoveryPage({ phrase, name, identity, address, logo: ParityLogo }));
  }
}
