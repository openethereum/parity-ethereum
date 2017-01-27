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

import DappsStore from '../dappsStore';
import ModalStore from '../modalStore';

import Button from '../Button';
import Modal from '../Modal';

import styles from '../Modal/modal.css';

const HEADERS = [
  'Error During Update',
  'Confirm Application Update',
  'Waiting for Signer Confirmation',
  'Waiting for Transaction Receipt',
  'Update Completed'
];
const STEP_ERROR = 0;
const STEP_CONFIRM = 1;
const STEP_SIGNER = 2;
const STEP_TXRECEIPT = 3;
const STEP_DONE = 4;

@observer
export default class ModalUpdate extends Component {
  dappsStore = DappsStore.instance();
  modalStore = ModalStore.instance();

  render () {
    if (!this.modalStore.showingUpdate) {
      return null;
    }

    return (
      <Modal
        buttons={ this.renderButtons() }
        error={ this.modalStore.errorUpdate }
        header={ HEADERS[this.modalStore.stepUpdate] }
      >
        { this.renderStep() }
      </Modal>
    );
  }

  renderButtons () {
    switch (this.modalStore.stepUpdate) {
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
            key='delete'
            label='Yes, Update'
            warning
            onClick={ this.onClickYes }
          />
        ];
      default:
        return null;
    }
  }

  renderStep () {
    switch (this.modalStore.stepUpdate) {
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
          Your application metadata has been updated in the registry.
        </div>
      </div>
    );
  }

  renderStepConfirm () {
    return (
      <div>
        <div className={ styles.section }>
          You are about to update the application details in the registry, the details of these updates are given below. Please note that each update will generate a seperate transaction.
        </div>
        <div className={ styles.section }>
          <div className={ styles.heading }>
            Application identifier
          </div>
          <div>
            { this.dappsStore.wipApp.id }
          </div>
        </div>
        { this.renderChanges() }
      </div>
    );
  }

  renderChanges () {
    return ['content', 'image', 'manifest']
      .filter((type) => this.dappsStore.wipApp[`${type}Changed`])
      .map((type) => {
        return (
          <div className={ styles.section } key={ `${type}Update` }>
            <div className={ styles.heading }>
              Updates to { type } hash
            </div>
            <div>
              <div>{ this.dappsStore.wipApp[`${type}Hash`] || '(removed)' }</div>
              <div className={ styles.hint }>
                { this.dappsStore.wipApp[`${type}Url`] || 'current url to be removed from registry' }
              </div>
            </div>
          </div>
        );
      });
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
    this.modalStore.hideUpdate();
  }

  onClickYes = () => {
    this.modalStore.doUpdate();
  }
}
