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
import { observer } from 'mobx-react';

import { api } from '../parity';
import DappsStore from '../dappsStore';
import ModalStore from '../modalStore';

import Button from '../Button';
import Modal from '../Modal';

import styles from '../Modal/modal.css';

const HEADERS = [
  'Error During Registration',
  'Confirm Application Registration',
  'Waiting for Signer Confirmation',
  'Waiting for Transaction Receipt',
  'Registration Completed'
];
const STEP_ERROR = 0;
const STEP_CONFIRM = 1;
const STEP_SIGNER = 2;
const STEP_TXRECEIPT = 3;
const STEP_DONE = 4;

@observer
export default class ModalRegister extends Component {
  dappsStore = DappsStore.instance();
  modalStore = ModalStore.instance();

  render () {
    if (!this.modalStore.showingRegister) {
      return null;
    }

    return (
      <Modal
        buttons={ this.renderButtons() }
        error={ this.modalStore.errorRegister }
        header={ HEADERS[this.modalStore.stepRegister] }
      >
        { this.renderStep() }
      </Modal>
    );
  }

  renderButtons () {
    switch (this.modalStore.stepRegister) {
      case STEP_ERROR:
      case STEP_DONE:
        return [
          <Button
            key='close'
            label='Close'
            onClick={ this.onClickClose }
          />
        ];
      case STEP_CONFIRM:
        return [
          <Button
            key='cancel'
            label='No, Cancel'
            onClick={ this.onClickClose }
          />,
          <Button
            key='register'
            label='Yes, Register'
            warning
            onClick={ this.onClickConfirmYes }
          />
        ];
      default:
        return null;
    }
  }

  renderStep () {
    switch (this.modalStore.stepRegister) {
      case STEP_CONFIRM:
        return this.renderStepConfirm();
      case STEP_SIGNER:
        return this.renderStepWait('Waiting for transaction confirmation in the Parity secure signer');
      case STEP_TXRECEIPT:
        return this.renderStepWait('Waiting for the transaction receipt from the network');
      case STEP_DONE:
        return this.renderStepCompleted();
      default:
        return null;
    }
  }

  renderStepCompleted () {
    return (
      <div>
        <div className={ styles.section }>
          Your application has been registered in the registry.
        </div>
      </div>
    );
  }

  renderStepConfirm () {
    return (
      <div>
        <div className={ styles.section }>
          You are about to register a new distributed application on the network, the details of this application is given below. This will require a non-refundable fee of { api.util.fromWei(this.dappsStore.fee).toFormat(3) }<small>ETH</small>.
        </div>
        <div className={ styles.section }>
          <div className={ styles.heading }>
            Selected owner account
          </div>
          <div className={ styles.account }>
            <img src={ api.util.createIdentityImg(this.dappsStore.currentAccount.address, 3) } />
            <div>{ this.dappsStore.currentAccount.name }</div>
            <div className={ styles.hint }>{ this.dappsStore.currentAccount.address }</div>
          </div>
        </div>
        <div className={ styles.section }>
          <div className={ styles.heading }>
            Unique assigned application identifier
          </div>
          <div>
            { this.dappsStore.wipApp.id }
          </div>
        </div>
      </div>
    );
  }

  renderStepWait (waitingFor) {
    return (
      <div>
        <div className={ styles.section }>
          { waitingFor }
        </div>
      </div>
    );
  }

  onClickClose = () => {
    this.modalStore.hideRegister();
  }

  onClickConfirmYes = () => {
    this.modalStore.doRegister();
  }
}
