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
import { observer } from 'mobx-react';
import DoneIcon from 'material-ui/svg-icons/action/done-all';
import CancelIcon from 'material-ui/svg-icons/content/clear';

import { Button, IdentityIcon, Modal } from '~/ui';
import RadioButtons from '~/ui/Form/RadioButtons';
import { nullableProptype } from '~/util/proptypes';

import styles from './verification.css';

const methods = {
  sms: {
    label: 'SMS Verification', key: 0, value: 'sms',
    description: (<p className={ styles.noSpacing }>It will be stored on the blockchain that you control a phone number (not <em>which</em>).</p>)
  },
  email: {
    label: 'E-mail Verification', key: 1, value: 'email',
    description: (<p className={ styles.noSpacing }>The hash of the e-mail address you prove control over will be stored on the blockchain.</p>)
  }
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
    store: nullableProptype(PropTypes.object.isRequired),
    account: PropTypes.string.isRequired,
    onSelectMethod: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  }

  static phases = { // mapping (store steps -> steps)
    [LOADING]: 1, [QUERY_DATA]: 1,
    [POSTING_REQUEST]: 2, [POSTED_REQUEST]: 2, [REQUESTING_CODE]: 2,
    [QUERY_CODE]: 3,
    [POSTING_CONFIRMATION]: 4, [POSTED_CONFIRMATION]: 4,
    [DONE]: 5
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
        steps={ ['Method', 'Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        waiting={ error ? [] : [ 2, 4 ] }
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

    if (phase === 5) {
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
      case 1:
        action = store.sendRequest;
        break;
      case 2:
        action = store.queryCode;
        break;
      case 3:
        action = store.sendConfirmation;
        break;
      case 4:
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

    const { method } = this.state;
    if (phase === 0) {
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
      fee, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setCode
    } = this.props.store;

    switch (phase) {
      case 1:
        if (step === LOADING) {
          return (<p>Loading verification data.</p>);
        }

        const { setConsentGiven } = this.props.store;

        const fields = [];
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
            method={ method } fields={ fields }
            fee={ fee } isVerified={ isVerified } hasRequested={ hasRequested }
            setConsentGiven={ setConsentGiven }
          />
        );

      case 2:
        return (
          <SendRequest step={ step } tx={ requestTx } />
        );

      case 3:
        let receiver, hint;
        if (method === 'sms') {
          receiver = this.props.store.number;
          hint = 'Enter the code you received via SMS.';
        } else if (method === 'email') {
          receiver = this.props.store.email;
          hint = 'Enter the code you received via e-mail.';
        }
        return (
          <QueryCode
            receiver={ receiver }
            hint={ hint }
            isCodeValid={ isCodeValid }
            setCode={ setCode }
          />
        );

      case 4:
        return (
          <SendConfirmation step={ step } tx={ confirmationTx } />
        );

      case 5:
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
