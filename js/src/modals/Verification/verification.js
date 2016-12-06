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
import { observer } from 'mobx-react';
import DoneIcon from 'material-ui/svg-icons/action/done-all';
import CancelIcon from 'material-ui/svg-icons/content/clear';

import { Button, IdentityIcon, Modal } from '~/ui';
import RadioButtons from '~/ui/Form/RadioButtons';
import { nullableProptype } from '~/util/proptypes';

const methods = {
  sms: { label: 'SMS Verification', key: 0, value: 'sms' },
  email: { label: 'E-mail Verification', key: 1, value: 'email' }
};

import {
  LOADING,
  QUERY_DATA,
  POSTING_REQUEST, POSTED_REQUEST,
  REQUESTING_CODE, QUERY_CODE,
  POSTING_CONFIRMATION, POSTED_CONFIRMATION,
  DONE
} from './store';

import GatherData from './GatherData';
import SendRequest from './SendRequest';
import QueryCode from './QueryCode';
import SendConfirmation from './SendConfirmation';
import Done from './Done';

@observer
export default class Verification extends Component {
  static propTypes = {
    store: nullableProptype(PropTypes.object).isRequired,
    account: PropTypes.string.isRequired,
    onSelectMethod: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  }

  static phases = { // mapping (store steps -> steps)
    [LOADING]: 1,
    [QUERY_DATA]: 2,
    [POSTING_REQUEST]: 3, [POSTED_REQUEST]: 3, [REQUESTING_CODE]: 3,
    [QUERY_CODE]: 4,
    [POSTING_CONFIRMATION]: 5, [POSTED_CONFIRMATION]: 5,
    [DONE]: 6
  }

  state = {
    method: 'sms'
  };

  render () {
    const { store } = this.props;
    let phase = 0; let error = false; let isStepValid = true;

    if (store) {
      phase = Verification.phases[store.step];
      error = store.error;
      isStepValid = store.isStepValid;
    }

    return (
      <Modal
        actions={ this.renderDialogActions(phase, error, isStepValid) }
        title='verify your account'
        visible
        current={ phase }
        steps={ ['Method', 'Prepare', 'Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        waiting={ error ? [] : [ 1, 3, 5 ] }
      >
        { this.renderStep(phase, error) }
      </Modal>
    );
  }

  renderDialogActions (phase, error, isStepValid) {
    const { store, account, onClose } = this.props;

    const cancel = (
      <Button
        key='cancel' label='Cancel'
        icon={ <CancelIcon /> }
        onClick={ onClose }
      />
    );
    if (error) {
      return (<div>{ cancel }</div>);
    }

    if (phase === 6) {
      return (
        <div>
          { cancel }
          <Button
            key='done' label='Done'
            disabled={ !isStepValid }
            icon={ <DoneIcon /> }
            onClick={ onClose }
          />
        </div>
      );
    }

    let action = () => {};
    switch (phase) {
      case 0:
        action = () => {
          const { onSelectMethod } = this.props;
          const { method } = this.state;
          onSelectMethod(method);
        };
        break;
      case 2:
        action = store.sendRequest;
        break;
      case 3:
        action = store.queryCode;
        break;
      case 4:
        action = store.sendConfirmation;
        break;
      case 5:
        action = store.done;
        break;
    }

    return (
      <div>
        { cancel }
        <Button
          key='next' label='Next'
          disabled={ !isStepValid }
          icon={ <IdentityIcon address={ account } button /> }
          onClick={ action }
        />
      </div>
    );
  }

  renderStep (phase, error) {
    if (error) {
      return (<p>{ error }</p>);
    }

    if (phase === 0) {
      const { method } = this.state;
      const values = Object.values(methods);
      const value = values.findIndex((v) => v.value === method);
      return (
        <RadioButtons
          value={ value < 0 ? 0 : value }
          values={ values }
          onChange={ this.selectMethod }
        />
      );
    }

    const {
      step,
      fee, number, isNumberValid, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setCode
    } = this.props.store;

    switch (phase) {
      case 1:
        return (
          <p>Loading Verification.</p>
        );

      case 2:
        const { method } = this.state;
        const { setConsentGiven } = this.props.store;

        const fields = []
        if (method === 'sms') {
          fields.push({
            key: 'number',
            label: 'phone number in international format',
            hint: 'the SMS will be sent to this number',
            error: this.props.store.isNumberValid ? null : 'invalid number',
            onChange: this.props.store.setNumber
          });
        } else if (method === 'email') {
          fields.push({
            key: 'email',
            label: 'email address',
            hint: 'the code will be sent to this address',
            error: this.props.store.isEmailValid ? null : 'invalid email',
            onChange: this.props.store.setEmail
          });
        }

        return (
          <GatherData
            fee={ fee } isNumberValid={ isNumberValid }
            isVerified={ isVerified } hasRequested={ hasRequested }
            setNumber={ setNumber } setConsentGiven={ setConsentGiven }
          />
        );

      case 3:
        return (
          <SendRequest step={ step } tx={ requestTx } />
        );

      case 4:
        return (
          <QueryCode
            number={ number } fee={ fee } isCodeValid={ isCodeValid }
            setCode={ setCode }
          />
        );

      case 5:
        return (
          <SendConfirmation step={ step } tx={ confirmationTx } />
        );

      case 6:
        return (
          <Done />
        );

      default:
        return null;
    }
  }

  selectMethod = (choice, i) => {
    this.setState({ method: choice.value });
  }
}
