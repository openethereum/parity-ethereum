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
import { observer } from 'mobx-react';
import { observable } from 'mobx';
import DoneIcon from 'material-ui/svg-icons/action/done-all';
import CancelIcon from 'material-ui/svg-icons/content/clear';

import { Button, IdentityIcon, Modal } from '~/ui';
import RadioButtons from '~/ui/Form/RadioButtons';

import SMSVerificationStore from './sms-store';
import EmailVerificationStore from './email-store';

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
class Verification extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,
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

  @observable store = null;

  render () {
    const store = this.store;
    let phase = 0;
    let error = false;
    let isStepValid = true;

    if (store) {
      phase = Verification.phases[store.step];
      error = store.error;
      isStepValid = store.isStepValid;
    }

    return (
      <Modal
        actions={ this.renderDialogActions(phase, error, isStepValid) }
        current={ phase }
        steps={ ['Method', 'Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        title='verify your account'
        visible
        waiting={ error ? [] : [ 2, 4 ] }
      >
        { this.renderStep(phase, error) }
      </Modal>
    );
  }

  renderDialogActions (phase, error, isStepValid) {
    const { account, onClose } = this.props;
    const store = this.store;

    const cancel = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label='Cancel'
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
            disabled={ !isStepValid }
            icon={ <DoneIcon /> }
            key='done'
            label='Done'
            onClick={ onClose }
          />
        </div>
      );
    }

    let action = () => {};

    switch (phase) {
      case 0:
        action = () => {
          const { method } = this.state;

          this.onSelectMethod(method);
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
          disabled={ !isStepValid }
          icon={
            <IdentityIcon
              address={ account }
              button
            />
          }
          key='next'
          label='Next'
          onClick={ action }
        />
      </div>
    );
  }

  renderStep (phase, error) {
    if (error) {
      return (
        <p>{ error }</p>
      );
    }

    const { method } = this.state;

    if (phase === 0) {
      const values = Object.values(methods);
      const value = values.findIndex((v) => v.value === method);

      return (
        <RadioButtons
          onChange={ this.selectMethod }
          value={ value < 0 ? 0 : value }
          values={ values }
        />
      );
    }

    const {
      step,
      isServerRunning, fee, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setCode
    } = this.store;

    switch (phase) {
      case 1:
        if (step === LOADING) {
          return (<p>Loading verification data.</p>);
        }

        const { setConsentGiven } = this.store;
        const fields = [];

        if (method === 'sms') {
          fields.push({
            key: 'number',
            label: 'phone number in international format',
            hint: 'the SMS will be sent to this number',
            error: this.store.isNumberValid ? null : 'invalid number',
            onChange: this.store.setNumber
          });
        } else if (method === 'email') {
          fields.push({
            key: 'email',
            label: 'email address',
            hint: 'the code will be sent to this address',
            error: this.store.isEmailValid ? null : 'invalid email',
            onChange: this.store.setEmail
          });
        }

        return (
          <GatherData
            fee={ fee }
            hasRequested={ hasRequested }
            isServerRunning={ isServerRunning }
            isVerified={ isVerified }
            method={ method } fields={ fields }
            setConsentGiven={ setConsentGiven }
          />
        );

      case 2:
        return (
          <SendRequest
            step={ step }
            tx={ requestTx }
          />
        );

      case 3:
        let receiver;
        let hint;

        if (method === 'sms') {
          receiver = this.store.number;
          hint = 'Enter the code you received via SMS.';
        } else if (method === 'email') {
          receiver = this.store.email;
          hint = 'Enter the code you received via e-mail.';
        }
        return (
          <QueryCode
            hint={ hint }
            isCodeValid={ isCodeValid }
            receiver={ receiver }
            setCode={ setCode }
          />
        );

      case 4:
        return (
          <SendConfirmation
            step={ step }
            tx={ confirmationTx }
          />
        );

      case 5:
        return (
          <Done />
        );

      default:
        return null;
    }
  }

  onSelectMethod = (name) => {
    const { api } = this.context;
    const { account, isTest } = this.props;

    if (name === 'sms') {
      this.store = new SMSVerificationStore(api, account, isTest);
    } else if (name === 'email') {
      this.store = new EmailVerificationStore(api, account, isTest);
    }
  }

  selectMethod = (choice, i) => {
    this.setState({ method: choice.value });
  }
}

const mapStateToProps = (state) => ({
  isTest: state.nodeStatus.isTest
});

export default connect(
  mapStateToProps,
  null
)(Verification);
