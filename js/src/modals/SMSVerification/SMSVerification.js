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

import VerificationStore from './store';
const {
  GATHERING_DATA, GATHERED_DATA,
  POSTING_REQUEST, POSTED_REQUEST,
  REQUESTING_SMS, REQUESTED_SMS,
  POSTING_CONFIRMATION, POSTED_CONFIRMATION,
  DONE
} = VerificationStore;

import GatherData from './GatherData';
import SendRequest from './SendRequest';
import QueryCode from './QueryCode';
import SendConfirmation from './SendConfirmation';
import Done from './Done';

@observer
export default class SMSVerification extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string,
    onClose: PropTypes.func.isRequired
  }

  static uiSteps = { // mapping (store steps -> steps)
    [GATHERING_DATA]: 0, [GATHERED_DATA]: 0,
    [POSTING_REQUEST]: 1, [POSTED_REQUEST]: 1, [REQUESTING_SMS]: 1,
    [REQUESTED_SMS]: 2,
    [POSTING_CONFIRMATION]: 3, [POSTED_CONFIRMATION]: 3,
    [DONE]: 4
  }

  state = {
    store: null
  }

  componentDidMount () {
    const { api } = this.context;
    const { account } = this.props;

    const store = new VerificationStore(api, account);
    this.setState({ store }, () => {
      store.gatherData();
    });
  }

  render () {
    const { store } = this.state;
    if (!store) return null;
    const step = SMSVerification.uiSteps[store.step];
    const { error, isStepValid } = store;

    return (
      <Modal
        actions={ this.renderDialogActions(step, error, isStepValid) }
        title='verify your account via SMS'
        visible scroll
        current={ step }
        steps={ ['Enter Data', 'Request', 'Enter Code', 'Confirm', 'Done!'] }
        waiting={ [ 1, 3 ] }
      >
        { this.renderStep(step, error) }
      </Modal>
    );
  }

  renderDialogActions (step, error, isStepValid) {
    const { onClose, account } = this.props;
    const { store } = this.state;

    const cancel = (
      <Button
        key='cancel' label='Cancel'
        icon={ <ContentClear /> }
        onClick={ onClose }
      />
    );
    if (error) return (<div>{ cancel }</div>);

    if (step === 4) {
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
    if (step === 3) {
      action = store.done;
    } else if (step === 2) {
      action = store.sendConfirmation;
    } else if (step === 1) {
      action = store.queryCode;
    } else if (step === 0) {
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

  renderStep (step, error) {
    if (error) return (<p>{ error }</p>);

    const {
      fee, isNumberValid, isVerified, hasRequested,
      requestTx, isCodeValid, confirmationTx,
      setNumber, setConsentGiven, setCode
    } = this.state.store;

    if (step === 4) {
      return (<Done />);
    }
    if (step === 3) {
      return (<SendConfirmation step={ step } tx={ confirmationTx } />);
    }
    if (step === 2) {
      return (<QueryCode fee={ fee } isCodeValid={ isCodeValid } setCode={ setCode } />);
    }
    if (step === 1) {
      return (<SendRequest step={ step } tx={ requestTx } />);
    }
    if (step === 0) {
      const { setNumber, setConsentGiven } = this.state.store;
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
