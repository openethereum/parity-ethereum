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
import styles from './buttonBar.css';

@observer
export default class ButtonBar extends Component {
  dappsStore = DappsStore.instance();
  modalStore = ModalStore.instance();

  render () {
    let buttons = [];

    if (this.dappsStore.isEditing || this.dappsStore.isNew) {
      buttons = [
        <Button
          key='cancel'
          label='Cancel'
          warning
          onClick={ this.onCancelClick }
        />,
        <Button
          key='save'
          label={ this.dappsStore.isNew ? 'Register' : 'Update' }
          disabled={ !this.dappsStore.canSave }
          onClick={ this.onSaveClick }
        />
      ];
    } else {
      buttons = [
        <Button
          key='delete'
          label='Delete'
          warning
          disabled={ !this.dappsStore.currentApp || (!this.dappsStore.currentApp.isOwner && !this.dappsStore.isContractOwner) }
          onClick={ this.onDeleteClick }
        />,
        <Button
          key='edit'
          label='Edit'
          disabled={ !this.dappsStore.currentApp || !this.dappsStore.currentApp.isOwner }
          onClick={ this.onEditClick }
        />,
        <Button
          key='new'
          label='New'
          onClick={ this.onNewClick }
        />
      ];
    }

    return (
      <div className={ styles.buttonbar }>
        { buttons }
      </div>
    );
  }

  onCancelClick = () => {
    if (this.dappsStore.isEditing) {
      this.dappsStore.setEditing(false);
    } else {
      this.dappsStore.setNew(false);
    }
  }

  onDeleteClick = () => {
    this.modalStore.showDelete();
  }

  onEditClick = () => {
    this.dappsStore.setEditing(true);
  }

  onNewClick = () => {
    this.dappsStore.setNew(true);
  }

  onSaveClick = () => {
    if (this.dappsStore.isEditing) {
      this.modalStore.showUpdate();
    } else {
      this.modalStore.showRegister();
    }
  }
}
