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

import { Button, IdentityIcon, Modal } from '../../ui';

import {
  LOADING,
  QUERY_DATA,
  POSTING_REQUEST, POSTED_REQUEST,
  REQUESTING_SMS, QUERY_CODE,
  POSTING_CONFIRMATION, POSTED_CONFIRMATION,
  DONE
} from './store';

import GatherData from './GatherData';
import SendRequest from './SendRequest';
import QueryCode from './QueryCode';
import SendConfirmation from './SendConfirmation';
import Done from './Done';

@observer
export default class SMSVerification extends Component {
  static propTypes = {
    store: PropTypes.any.isRequired,
    account: PropTypes.string.isRequired,
    onClose: PropTypes.func.isRequired
  }

  static phases = { // mapping (store steps -> steps)
    [LOADING]: 0,
    [QUERY_DATA]: 1,
    [POSTING_REQUEST]: 2, [POSTED_REQUEST]: 2, [REQUESTING_SMS]: 2,
    [QUERY_CODE]: 3,
    [POSTING_CONFIRMATION]: 4, [POSTED_CONFIRMATION]: 4,
    [DONE]: 5
  }

  render () {
    const phase = SMSVerification.phases[this.props.store.step];
    const { error, isStepValid } = this.props.store;

    return (
      <Modal
        actions={ this.renderDialogActions(phase, error, isStepValid) }
        title='verify your account via SMS'
        visible scroll
        current={ phase }
        steps={ ['Prepare', 'Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        waiting={ error ? [] : [ 0, 2, 4 ] }
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

    const {
      step,
      fee, number, isNumberValid, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setNumber, setConsentGiven, setCode
    } = this.props.store;

    switch (phase) {
      case 0:
        return (
          <p>Loading SMS Verification.</p>
        );

      case 1:
        const { setNumber, setConsentGiven } = this.props.store;
        return (
          <GatherData
            fee={ fee } isNumberValid={ isNumberValid }
            isVerified={ isVerified } hasRequested={ hasRequested }
            setNumber={ setNumber } setConsentGiven={ setConsentGiven }
          />
        );

      case 2:
        return (
          <SendRequest step={ step } tx={ requestTx } />
        );

      case 3:
        return (
          <QueryCode
            number={ number } fee={ fee } isCodeValid={ isCodeValid }
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
}
