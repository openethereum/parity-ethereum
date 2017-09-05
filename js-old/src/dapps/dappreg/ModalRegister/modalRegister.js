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

import Modal from '../Modal';

import styles from '../Modal/modal.css';

@observer
export default class ModalRegister extends Component {
  static propTypes = {
    dappId: PropTypes.string.isRequired,
    onClose: PropTypes.func.isRequired,
    onRegister: PropTypes.func.isRequired
  };

  dappsStore = DappsStore.get();

  render () {
    const { onClose, onRegister } = this.props;
    const actions = [
      { type: 'close', label: 'No, Cancel' },
      { type: 'confirm', label: 'Yes, Register', warning: true }
    ];

    return (
      <Modal
        actions={ actions }
        header='Confirm Application Registration'
        onClose={ onClose }
        onConfirm={ onRegister }
        secondary
      >
        <div className={ styles.section }>
          You are about to register a new decentralized application on the network, the details of
          this application is given below. This will require a non-refundable fee
          of { api.util.fromWei(this.dappsStore.fee).toFormat(3) } <small>ETH</small>
        </div>
        <div className={ styles.section }>
          <div className={ styles.heading }>
            Unique assigned application identifier
          </div>
          <div>
            { this.props.dappId }
          </div>
        </div>
      </Modal>
    );
  }
}
