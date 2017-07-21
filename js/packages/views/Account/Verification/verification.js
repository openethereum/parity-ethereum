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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { observer } from 'mobx-react';
import { observable } from 'mobx';

import { Button, IdentityIcon, Portal, RadioButtons } from '@parity/ui';
import { CancelIcon, DoneIcon } from '@parity/ui/Icons';

import SMSVerificationStore from './sms-store';
import EmailVerificationStore from './email-store';

import styles from './verification.css';

const METHODS = {
  sms: {
    label: (
      <FormattedMessage
        id='verification.types.sms.label'
        defaultMessage='SMS Verification'
      />
    ),
    key: 'sms',
    value: 'sms',
    description: (
      <p className={ styles.noSpacing }>
        <FormattedMessage
          id='verification.types.sms.description'
          defaultMessage='It will be stored on the blockchain that you control a phone number (not <em>which</em>).'
        />
      </p>
    )
  },
  email: {
    label: (
      <FormattedMessage
        id='verification.types.email.label'
        defaultMessage='E-mail Verification'
      />
    ),
    key: 'email',
    value: 'email',
    description: (
      <p className={ styles.noSpacing }>
        <FormattedMessage
          id='verification.types.email.description'
          defaultMessage='The hash of the e-mail address you prove control over will be stored on the blockchain.'
        />
      </p>
    )
  }
};

const STEPS = [
  <FormattedMessage
    id='verification.steps.method'
    defaultMessage='Method'
  />,
  <FormattedMessage
    id='verification.steps.data'
    defaultMessage='Enter Data'
  />,
  <FormattedMessage
    id='verification.steps.request'
    defaultMessage='Request'
  />,
  <FormattedMessage
    id='verification.steps.code'
    defaultMessage='Enter Code'
  />,
  <FormattedMessage
    id='verification.steps.confirm'
    defaultMessage='Confirm'
  />,
  <FormattedMessage
    id='verification.steps.completed'
    defaultMessage='Completed'
  />
];

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
    const { onClose } = this.props;
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
      <Portal
        activeStep={ phase }
        busySteps={
          error
            ? []
            : [ 2, 4 ]
        }
        buttons={ this.renderDialogActions(phase, error, isStepValid) }
        onClose={ onClose }
        open
        steps={ STEPS }
        title={
          <FormattedMessage
            id='verification.title'
            defaultMessage='verify your account'
          />
        }
      >
        { this.renderStep(phase, error) }
      </Portal>
    );
  }

  renderDialogActions (phase, error, isStepValid) {
    const { account, onClose } = this.props;
    const store = this.store;

    const cancelButton = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='verification.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ onClose }
      />
    );

    if (error) {
      return cancelButton;
    }

    if (phase === 5) {
      return [
        cancelButton,
        <Button
          disabled={ !isStepValid }
          icon={ <DoneIcon /> }
          key='done'
          label={
            <FormattedMessage
              id='verification.button.done'
              defaultMessage='Done'
            />
          }
          onClick={ onClose }
        />
      ];
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

    return [
      cancelButton,
      <Button
        disabled={ !isStepValid }
        icon={
          <IdentityIcon
            address={ account }
            button
          />
        }
        key='next'
        label={
          <FormattedMessage
            id='verification.button.next'
            defaultMessage='Next'
          />
        }
        onClick={ action }
      />
    ];
  }

  renderStep (phase, error) {
    if (error) {
      return (
        <p>{ error }</p>
      );
    }

    const { method } = this.state;

    if (phase === 0) {
      return (
        <RadioButtons
          name='verificationType'
          onChange={ this.selectMethod }
          value={ method || 'sms' }
          values={ Object.values(METHODS) }
        />
      );
    }

    const {
      step,
      isServerRunning, isAbleToRequest, fee, accountIsVerified, accountHasRequested,
      requestTx, isCodeValid, confirmationTx,
      setCode
    } = this.store;

    switch (phase) {
      case 1:
        if (step === LOADING) {
          return (
            <p>
              <FormattedMessage
                id='verification.loading'
                defaultMessage='Loading verification data.'
              />
            </p>
          );
        }

        const { setConsentGiven } = this.store;
        const fields = [];

        if (method === 'sms') {
          fields.push({
            key: 'number',
            label: (
              <FormattedMessage
                id='verification.gatherData.phoneNumber.label'
                defaultMessage='phone number in international format'
              />
            ),
            hint: (
              <FormattedMessage
                id='verification.gatherData.phoneNumber.hint'
                defaultMessage='the SMS will be sent to this number'
              />
            ),
            error: this.store.isNumberValid
              ? null
              : (
                <FormattedMessage
                  id='verification.gatherDate.phoneNumber.error'
                  defaultMessage='invalid number'
                />
              ),
            onChange: this.store.setNumber
          });
        } else if (method === 'email') {
          fields.push({
            key: 'email',
            label: (
              <FormattedMessage
                id='verification.gatherData.email.label'
                defaultMessage='e-mail address'
              />
            ),
            hint: (
              <FormattedMessage
                id='verification.gatherData.email.hint'
                defaultMessage='the code will be sent to this address'
              />
            ),
            error: this.store.isEmailValid
              ? null
              : (
                <FormattedMessage
                  id='verification.gatherDate.email.error'
                  defaultMessage='invalid e-mail'
                />
              ),
            onChange: this.store.setEmail
          });
        }

        return (
          <GatherData
            fee={ fee }
            accountHasRequested={ accountHasRequested }
            isServerRunning={ isServerRunning }
            isAbleToRequest={ isAbleToRequest }
            accountIsVerified={ accountIsVerified }
            method={ method }
            fields={ fields }
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
          hint = (
            <FormattedMessage
              id='verification.sms.enterCode'
              defaultMessage='Enter the code you received via SMS.'
            />
          );
        } else if (method === 'email') {
          receiver = this.store.email;
          hint = (
            <FormattedMessage
              id='verification.email.enterCode'
              defaultMessage='Enter the code you received via e-mail.'
            />
          );
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

  selectMethod = (event, method) => {
    this.setState({ method });
  }
}

const mapStateToProps = (state) => ({
  isTest: state.nodeStatus.isTest
});

export default connect(
  mapStateToProps,
  null
)(Verification);
