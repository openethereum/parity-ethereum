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

import React, { Component, PropTypes } from 'react';
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

@observer
export default class ModalRegister extends Component {
  static propTypes = {
    dappId: PropTypes.string.isRequired,
    onClose: PropTypes.func.isRequired
  };

  dappsStore = DappsStore.instance();
  modalStore = ModalStore.instance();

  render () {
    return (
      <Modal
        buttons={ this.renderButtons() }
        error={ this.modalStore.errorRegister }
        header={ HEADERS[this.modalStore.stepRegister] }
      >
        { this.renderConfirm() }
      </Modal>
    );
  }

  renderButtons () {
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
  }

  renderConfirm () {
    return (
      <div>
        <div className={ styles.section }>
          You are about to register a new distributed application on the network, the details of this application is given below. This will require a non-refundable fee of { api.util.fromWei(this.dappsStore.fee).toFormat(3) }<small>ETH</small>.
        </div>
        <div className={ styles.section }>
          <div className={ styles.heading }>
            Unique assigned application identifier
          </div>
          <div>
            { this.props.dappId }
          </div>
        </div>
      </div>
    );
  }
  onClickClose = () => {
    this.props.onClose();
  }

  onClickConfirmYes = () => {
    this.modalStore.doRegister();
  }
}
