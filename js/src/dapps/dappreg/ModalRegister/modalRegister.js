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
        header='Confirm Application Registration'>
        <p>
          You are about to register a new distributed application on the network, the details of this application is given below. This will require a non-refundable fee of { api.util.fromWei(this.dappsStore.fee).toFormat(3) }<small>ETH</small>.
        </p>
        <p className={ styles.center }>
          <div className={ styles.heading }>
            Selected owner account
          </div>
          <div className={ styles.account }>
            <img src={ api.util.createIdentityImg(this.dappsStore.currentAccount.address, 3) } />
            <div>{ this.dappsStore.currentAccount.name }</div>
            <div className={ styles.address }>{ this.dappsStore.currentAccount.address }</div>
          </div>
        </p>
        <p className={ styles.center }>
          <div className={ styles.heading }>
            Unique assigned application identifier
          </div>
          <div>
            { this.dappsStore.wipApp.id }
          </div>
        </p>
      </Modal>
    );
  }

  renderButtons () {
    return [
      <Button
        key='cancel'
        label='No, Cancel'
        onClick={ this.onClickNo } />,
      <Button
        key='register'
        label='Yes, Register'
        warning
        onClick={ this.onClickYes } />
    ];
  }

  onClickNo = () => {
    this.modalStore.hideRegister();
  }

  onClickYes = () => {
  }
}
