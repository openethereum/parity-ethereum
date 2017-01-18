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
import { connect } from 'react-redux';
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';
import PrintIcon from 'material-ui/svg-icons/action/print';

import { Button, Modal } from '~/ui';

import { NewAccount, AccountDetails } from '../CreateAccount';

import Completed from './Completed';
import TnC from './TnC';
import Welcome from './Welcome';

import { createIdentityImg } from '~/api/util/identity';
import print from '../CreateAccount/print';
import recoveryPage from '../CreateAccount/recovery-page.ejs';
import ParityLogo from '../../../assets/images/parity-logo-black-no-text.svg';

const STAGE_NAMES = ['welcome', 'terms', 'new account', 'recovery', 'completed'];

class FirstRun extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    hasAccounts: PropTypes.bool.isRequired,
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  }

  state = {
    stage: 0,
    name: '',
    address: '',
    password: '',
    phrase: '',
    canCreate: false,
    hasAcceptedTnc: false
  }

  render () {
    const { visible } = this.props;
    const { stage } = this.state;

    if (!visible) {
      return null;
    }

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ STAGE_NAMES }
        visible
      >
        { this.renderStage() }
      </Modal>
    );
  }

  renderStage () {
    const { address, name, phrase, stage, hasAcceptedTnc } = this.state;

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
            onChange={ this.onChangeDetails }
          />
        );
      case 3:
        return (
          <AccountDetails
            address={ address }
            name={ name }
            phrase={ phrase }
          />
        );
      case 4:
        return (
          <Completed />
        );
    }
  }

  renderDialogActions () {
    const { hasAccounts } = this.props;
    const { canCreate, stage, hasAcceptedTnc } = this.state;

    switch (stage) {
      case 0:
        return (
          <Button
            icon={ <NavigationArrowForward /> }
            label='Next'
            onClick={ this.onNext }
          />
        );

      case 1:
        return (
          <Button
            disabled={ !hasAcceptedTnc }
            icon={ <NavigationArrowForward /> }
            label='Next'
            onClick={ this.onNext }
          />
        );

      case 2:
        const buttons = [
          <Button
            icon={ <ActionDone /> }
            label='Create'
            key='create'
            disabled={ !canCreate }
            onClick={ this.onCreate }
          />
        ];
        if (hasAccounts) {
          buttons.unshift(
            <Button
              icon={ <NavigationArrowForward /> }
              label='Skip'
              key='skip'
              onClick={ this.skipAccountCreation }
            />
          );
        }
        return buttons;

      case 3:
        return [
          <Button
            icon={ <PrintIcon /> }
            label='Print Phrase'
            onClick={ this.printPhrase }
          />,
          <Button
            icon={ <NavigationArrowForward /> }
            label='Next'
            onClick={ this.onNext }
          />
        ];

      case 4:
        return (
          <Button
            icon={ <ActionDoneAll /> }
            label='Close'
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

  onChangeDetails = (valid, { name, address, password, phrase }) => {
    this.setState({
      canCreate: valid,
      name: name,
      address: address,
      password: password,
      phrase: phrase
    });
  }

  onCreate = () => {
    const { api } = this.context;
    const { name, phrase, password } = this.state;

    this.setState({
      canCreate: false
    });

    return api.parity
      .newAccountFromPhrase(phrase, password)
      .then((address) => api.parity.setAccountName(address, name))
      .then(() => {
        this.onNext();
      })
      .catch((error) => {
        console.error('onCreate', error);

        this.setState({
          canCreate: true
        });

        this.newError(error);
      });
  }

  skipAccountCreation = () => {
    this.setState({ stage: this.state.stage + 2 });
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

function mapStateToProps (state) {
  return { hasAccounts: state.personal.hasAccounts };
}

export default connect(mapStateToProps, null)(FirstRun);
