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

import { createIdentityImg } from '@parity/api/util/identity';
import { newError } from '@parity/shared/redux/actions';
import Button from '@parity/ui/Button';
import Portal from '@parity/ui/Portal';
import { CheckIcon, DoneIcon, NextIcon, PrintIcon, ReplayIcon } from '@parity/ui/Icons';

import ParityLogo from '@parity/shared/assets/images/parity-logo-black-no-text.svg';
import { NewAccount, AccountDetails } from '@parity/dapp-accounts/src/CreateAccount';
import print from '@parity/dapp-accounts/src/CreateAccount/print';
import recoveryPage from '@parity/dapp-accounts/src/CreateAccount/recoveryPage.ejs';
import CreateStore from '@parity/dapp-accounts/src/CreateAccount/store';

import Completed from './Completed';
import TnC from './TnC';
import Welcome from './Welcome';

const STAGE_NAMES = [
  <FormattedMessage
    id='firstRun.title.welcome'
    defaultMessage='welcome'
  />,
  <FormattedMessage
    id='firstRun.title.terms'
    defaultMessage='terms'
  />,
  <FormattedMessage
    id='firstRun.title.newAccount'
    defaultMessage='new account'
  />,
  <FormattedMessage
    id='firstRun.title.recovery'
    defaultMessage='recovery'
  />,
  <FormattedMessage
    id='firstRun.title.confirmation'
    defaultMessage='confirmation'
  />,
  <FormattedMessage
    id='firstRun.title.completed'
    defaultMessage='completed'
  />
];
const BUTTON_LABEL_NEXT = (
  <FormattedMessage
    id='firstRun.button.next'
    defaultMessage='Next'
  />
);

@observer
class FirstRun extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    hasAccounts: PropTypes.bool.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired,
    visible: PropTypes.bool.isRequired,
    isTest: PropTypes.bool.isRequired
  }

  createStore = new CreateStore(this.context.api, {}, this.props.isTest, false);

  state = {
    stage: 0,
    hasAcceptedTnc: false
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.isTest !== this.props.isTest) {
      this.createStore.setIsTest(nextProps.isTest);
    }
  }

  render () {
    const { visible } = this.props;
    const { stage } = this.state;

    if (!visible) {
      return null;
    }

    return (
      <Portal
        buttons={ this.renderDialogActions() }
        activeStep={ stage }
        hideClose
        steps={ STAGE_NAMES }
        open
      >
        { this.renderStage() }
      </Portal>
    );
  }

  renderStage () {
    const { stage, hasAcceptedTnc } = this.state;

    switch (stage) {
      case 0:
        return (
          <Welcome />
        );
      case 1:
        return (
          <TnC
            hasAccepted={ hasAcceptedTnc }
            onAccept={ this.onAcceptTnC }
          />
        );
      case 2:
        return (
          <NewAccount
            newError={ this.props.newError }
            createStore={ this.createStore }
          />
        );
      case 3:
        return (
          <AccountDetails
            createStore={ this.createStore }
            withRequiredBackup
          />
        );
      case 4:
        return (
          <AccountDetails
            createStore={ this.createStore }
            isConfirming
          />
        );
      case 5:
        return (
          <Completed />
        );
    }
  }

  renderDialogActions () {
    const { hasAccounts } = this.props;
    const { stage, hasAcceptedTnc } = this.state;
    const { canCreate, phraseBackedUpError } = this.createStore;

    switch (stage) {
      case 0:
        return (
          <Button
            icon={ <NextIcon /> }
            key='next'
            label={ BUTTON_LABEL_NEXT }
            onClick={ this.onNext }
          />
        );

      case 1:
        return (
          <Button
            disabled={ !hasAcceptedTnc }
            icon={ <NextIcon /> }
            key='next'
            label={ BUTTON_LABEL_NEXT }
            onClick={ this.onNext }
          />
        );

      case 2:
        const buttons = [
          <Button
            disabled={ !canCreate }
            icon={ <NextIcon /> }
            key='next'
            label={ BUTTON_LABEL_NEXT }
            onClick={ this.onNext }
          />
        ];

        if (hasAccounts) {
          buttons.unshift(
            <Button
              icon={ <NextIcon /> }
              key='skip'
              label={
                <FormattedMessage
                  id='firstRun.button.skip'
                  defaultMessage='Skip'
                />
              }
              onClick={ this.skipAccountCreation }
            />
          );
        }
        return buttons;

      case 3:
        return [
          <Button
            icon={ <PrintIcon /> }
            key='print'
            label={
              <FormattedMessage
                id='firstRun.button.print'
                defaultMessage='Print Phrase'
              />
            }
            onClick={ this.printPhrase }
          />,
          <Button
            disabled={ !!phraseBackedUpError }
            icon={ <NextIcon /> }
            key='next'
            label={ BUTTON_LABEL_NEXT }
            onClick={ this.onConfirmPhraseBackup }
          />
        ];

      case 4:
        return [
          <Button
            icon={ <ReplayIcon /> }
            key='restart'
            label={
              <FormattedMessage
                id='firstRun.button.restart'
                defaultMessage='Start Over'
              />
            }
            onClick={ this.onStartOver }
          />,
          <Button
            icon={ <CheckIcon /> }
            key='create'
            label={
              <FormattedMessage
                id='firstRun.button.create'
                defaultMessage='Create'
              />
            }
            onClick={ this.onCreate }
          />
        ];

      case 5:
        return (
          <Button
            icon={ <DoneIcon /> }
            key='close'
            label={
              <FormattedMessage
                id='firstRun.button.close'
                defaultMessage='Close'
              />
            }
            onClick={ this.onClose }
          />
        );
    }
  }

  onClose = () => {
    const { onClose } = this.props;

    this.setState({
      stage: 0
    }, onClose);
  }

  onConfirmPhraseBackup = () => {
    this.createStore.clearPhrase();
    this.onNext();
  }

  onNext = () => {
    const { stage } = this.state;

    this.setState({
      stage: stage + 1
    });
  }

  onStartOver = () => {
    this.setState({
      stage: 2
    });
  }

  onAcceptTnC = () => {
    this.setState({
      hasAcceptedTnc: !this.state.hasAcceptedTnc
    });
  }

  onCreate = () => {
    this.createStore.setBusy(true);

    this.createStore.computeBackupPhraseAddress()
      .then(err => {
        if (err) {
          this.createStore.setBusy(false);
          return;
        }

        return this.createStore.createAccount()
          .then(() => {
            this.createStore.clearPhrase();
            this.createStore.setBusy(false);
            this.onNext();
          });
      })
      .catch((error) => {
        this.createStore.setBusy(false);
        this.props.newError(error);
      });
  }

  skipAccountCreation = () => {
    this.setState({ stage: this.state.stage + 3 });
  }

  printPhrase = () => {
    const { address, phrase, name } = this.createStore;
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
  const { hasAccounts } = state.personal;
  const { isTest } = state.nodeStatus;

  return {
    hasAccounts, isTest
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(FirstRun);
