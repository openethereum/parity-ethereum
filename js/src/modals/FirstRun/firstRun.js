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
import { Button, Portal } from '~/ui';
import { CheckIcon, DoneIcon, NextIcon, PrintIcon } from '~/ui/Icons';

import { NewAccount, AccountDetails } from '../CreateAccount';
import print from '../CreateAccount/print';
import recoveryPage from '../CreateAccount/recoveryPage.ejs';
import CreateStore from '../CreateAccount/store';

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
    visible: PropTypes.bool.isRequired
  }

  createStore = new CreateStore(this.context.api, {}, false);

  state = {
    stage: 0,
    hasAcceptedTnc: false
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
            store={ this.createStore }
          />
        );
      case 3:
        return (
          <AccountDetails store={ this.createStore } />
        );
      case 4:
        return (
          <Completed />
        );
    }
  }

  renderDialogActions () {
    const { hasAccounts } = this.props;
    const { stage, hasAcceptedTnc } = this.state;
    const { canCreate } = this.createStore;

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
            icon={ <NextIcon /> }
            key='next'
            label={ BUTTON_LABEL_NEXT }
            onClick={ this.onNext }
          />
        ];

      case 4:
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

  onNext = () => {
    const { stage } = this.state;

    this.setState({
      stage: stage + 1
    });
  }

  onAcceptTnC = () => {
    this.setState({
      hasAcceptedTnc: !this.state.hasAcceptedTnc
    });
  }

  onCreate = () => {
    this.createStore.setBusy(true);

    return this.createStore
      .createAccount()
      .then(() => {
        this.onNext();
        this.createStore.setBusy(false);
      })
      .catch((error) => {
        this.createStore.setBusy(false);
        this.props.newError(error);
      });
  }

  skipAccountCreation = () => {
    this.setState({ stage: this.state.stage + 2 });
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

  return {
    hasAccounts
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
