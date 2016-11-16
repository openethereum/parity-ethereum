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
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Button, IdentityIcon, Modal } from '../../ui';

import {
  GATHERING_DATA, GATHERED_DATA,
  POSTING_REQUEST, POSTED_REQUEST,
  REQUESTING_SMS, REQUESTED_SMS,
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
    [GATHERING_DATA]: 0, [GATHERED_DATA]: 0,
    [POSTING_REQUEST]: 1, [POSTED_REQUEST]: 1, [REQUESTING_SMS]: 1,
    [REQUESTED_SMS]: 2,
    [POSTING_CONFIRMATION]: 3, [POSTED_CONFIRMATION]: 3,
    [DONE]: 4
  }

  componentDidMount () {
    this.props.store.gatherData();
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
        steps={ ['Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        waiting={ error ? [] : [ 1, 3 ] }
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
        icon={ <ContentClear /> }
        onClick={ onClose }
      />
    );
    if (error) return (<div>{ cancel }</div>);

    if (phase === 4) {
      return (
        <div>
          { cancel }
          <Button
            key='done' label='Done'
            disabled={ !isStepValid }
            icon={ <ActionDoneAll /> }
            onClick={ onClose }
          />
        </div>
      );
    }

    let action;
    if (phase === 3) {
      action = store.done;
    } else if (phase === 2) {
      action = store.sendConfirmation;
    } else if (phase === 1) {
      action = store.queryCode;
    } else if (phase === 0) {
      action = store.sendRequest;
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
    if (error) return (<p>{ error }</p>);

    const {
      step,
      fee, number, isNumberValid, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setNumber, setConsentGiven, setCode
    } = this.props.store;

    if (phase === 4) {
      return (<Done />);
    }
    if (phase === 3) {
      return (<SendConfirmation step={ step } tx={ confirmationTx } />);
    }
    if (phase === 2) {
      return (
        <QueryCode
          number={ number } fee={ fee } isCodeValid={ isCodeValid }
          setCode={ setCode }
        />
      );
    }
    if (phase === 1) {
      return (<SendRequest step={ step } tx={ requestTx } />);
    }
    if (phase === 0) {
      const { setNumber, setConsentGiven } = this.props.store;
      return (
        <GatherData
          fee={ fee } isNumberValid={ isNumberValid }
          isVerified={ isVerified } hasRequested={ hasRequested }
          setNumber={ setNumber } setConsentGiven={ setConsentGiven }
        />
      );
    }

    return null;
  }
}
