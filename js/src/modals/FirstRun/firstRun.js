// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { Button, Modal } from '../../ui';

import { NewAccount, AccountDetails } from '../CreateAccount';

import Completed from './Completed';
import Welcome from './Welcome';

const STAGE_NAMES = ['welcome', 'new account', 'recovery', 'completed'];

export default class FirstRun extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    visible: PropTypes.bool,
    onClose: PropTypes.func.isRequired
  }

  state = {
    stage: 0,
    name: '',
    address: '',
    password: '',
    phrase: '',
    canCreate: false
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
        visible>
        { this.renderStage() }
      </Modal>
    );
  }

  renderStage () {
    const { address, name, phrase, stage } = this.state;

    switch (stage) {
      case 0:
        return (
          <Welcome />
        );
      case 1:
        return (
          <NewAccount
            onChange={ this.onChangeDetails } />
        );
      case 2:
        return (
          <AccountDetails
            address={ address }
            name={ name }
            phrase={ phrase } />
        );
      case 3:
        return (
          <Completed />
        );
    }
  }

  renderDialogActions () {
    const { canCreate, stage } = this.state;

    switch (stage) {
      case 0:
      case 2:
        return (
          <Button
            icon={ <NavigationArrowForward /> }
            label='Next'
            onClick={ this.onNext } />
        );

      case 1:
        return (
          <Button
            icon={ <ActionDone /> }
            label='Create'
            disabled={ !canCreate }
            onClick={ this.onCreate } />
        );

      case 3:
        return (
          <Button
            icon={ <ActionDoneAll /> }
            label='Close'
            onClick={ this.onClose } />
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

    return api.personal
      .newAccountFromPhrase(phrase, password)
      .then((address) => api.personal.setAccountName(address, name))
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

  newError = (error) => {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }
}
