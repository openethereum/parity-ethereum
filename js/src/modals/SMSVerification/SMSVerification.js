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
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { BusyStep, CompletedStep, Button, IdentityIcon, Modal } from '../../ui';
import { validateAddress, validateUint } from '../../util/validation';

import ABI from '../../contracts/abi/sms-verification.json';
const contract = '0xcE381B876A85A72303f7cA7b3a012f58F4CEEEeB';

import CheckIfCertified from './CheckIfCertified';
import GatherData from './GatherData';
import SendRequest from './SendRequest';

export default class SMSVerification extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    // store: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string,
    onClose: PropTypes.func.isRequired
  }

  state = {
    contract: null,
    step: 0,
    stepIsValid: false,
    data: {}
  }

  componentDidMount () {
    const { api } = this.context;

    this.setState({
      contract: api.newContract(ABI, contract)
    });
  }

  render () {
    const { step } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        title='verify your account via SMS'
        visible scroll
        current={ step }
        steps={ ['first step', 'second step', 'third step'] }
      >
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { onClose, account } = this.props;
    const { step, stepIsValid } = this.state;

    const cancel = (
      <Button
        key='cancel' label='Cancel'
        icon={ <ContentClear /> }
        onClick={ onClose }
      />
    );

    if (step === 2) {
      return (
        <div>
          { cancel }
          <Button
            key='done' label='Done'
            icon={ <ActionDoneAll /> }
            onClick={ onClose }
          />
        </div>
      );
    }

    return (
      <div>
        { cancel }
        <Button
          key='next' label='Next'
          disabled={ !stepIsValid }
          icon={ <IdentityIcon address={ account } button /> }
          onClick={ this.next }
        />
      </div>
    );
  }

  renderStep () {
    const { contract } = this.state;
    if (!contract) {
      return null;
    }

    const { step } = this.state;
    if (step === 2) {
      return this.renderThirdStep();
    } else if (step === 1) {
      return this.renderSecondStep();
    } else {
      return this.renderFirstStep();
    }
  }

  onDataIsValid = () => {
    this.setState({ stepIsValid: true });
  }
  onDataIsInvalid = () => {
    this.setState({ stepIsValid: false });
  }
  onData = (data) => {
    this.setState({
      data: Object.assign({}, this.state.data, data)
    });
  }

  next = () => {
    this.setState({ step: this.state.step + 1, stepIsValid: false });
  }

  renderFirstStep () {
    const { account } = this.props;
    const { contract } = this.state;

    return (
      <CheckIfCertified
        account={ account } contract={ contract }
        onIsCertified={ this.onDataIsInvalid }
        onIsNotCertified={ this.onDataIsValid }
      />
    );
  }

  renderSecondStep () {
    const { contract, data } = this.state;

    return (
      <GatherData
        contract={ contract } data={ data }
        onData={ this.onData }
        onDataIsValid={ this.onDataIsValid }
        onDataIsInvalid={ this.onDataIsInvalid }
      />
    );
  }

  renderThirdStep () {
    const { account } = this.props;
    const { contract, data } = this.state;

    return (
      <SendRequest
        account={ account } contract={ contract } data={ data }
        onData={ this.onData }
        onSuccess={ this.onDataIsValid }
        onError={ this.onDataIsInvalid }
      />
    );
  }
}
