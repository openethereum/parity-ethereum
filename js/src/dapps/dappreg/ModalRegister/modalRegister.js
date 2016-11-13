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
  'Waiting for Signer Confirmation'
];

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
        error={ this.modalStore.stepRegister === 0 }
        header={ HEADERS[this.modalStore.stepRegister] }>
        { this.renderStep() }
      </Modal>
    );
  }

  renderButtons () {
    switch (this.modalStore.stepRegister) {
      case 0:
        return [
          <Button
            key='close'
            label='Close'
            onClick={ this.onClickClose } />
        ];
      case 1:
        return [
          <Button
            key='cancel'
            label='No, Cancel'
            onClick={ this.onClickConfirmNo } />,
          <Button
            key='register'
            label='Yes, Register'
            warning
            onClick={ this.onClickConfirmYes } />
        ];
      default:
        return null;
    }
  }

  renderStep () {
    switch (this.modalStore.stepRegister) {
      case 0:
        return this.renderStepError();
      case 1:
        return this.renderStepConfirm();
      default:
        return null;
    }
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
            <div className={ styles.address }>{ this.dappsStore.currentAccount.address }</div>
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

  renderStepError () {
    return (
      <div>
        <div className={ styles.section }>
          Your transaction failed to complete sucessfully. The following error was returned:
        </div>
        <div className={ `${styles.section} ${styles.error}` }>
          { this.modalStore.errorRegister.toString() }
        </div>
      </div>
    );
  }

  onClickClose = () => {
    this.modalStore.hideRegister();
  }

  onClickConfirmNo = () => {
    this.onClickClose();
  }

  onClickConfirmYes = () => {
    this.modalStore.doRegister();
  }
}
