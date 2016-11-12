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

import Store from '../store';
import styles from './buttonBar.css';

@observer
export default class ButtonBar extends Component {
  store = Store.instance();

  render () {
    let buttons = [];

    if (this.store.isEditing || this.store.isNew) {
      buttons = [
        <button
          key='cancel'
          className={ styles.cancel }
          onClick={ this.onCancelClick }>
          Cancel
        </button>,
        <button
          key='save'
          disabled={ !this.store.canSave }
          onClick={ this.onSaveClick }>
          { this.store.isNew ? 'Register' : 'Update' }
        </button>
      ];
    } else {
      buttons = [
        <button
          key='delete'
          className={ styles.delete }
          disabled={ !this.store.currentApp.isOwner && !this.store.isContractOwner }
          onClick={ this.onDeleteClick }>
          Delete
        </button>,
        <button
          key='edit'
          disabled={ !this.store.currentApp.isOwner }
          onClick={ this.onEditClick }>
          Edit
        </button>,
        <button
          key='new'
          onClick={ this.onNewClick }>
          New
        </button>
      ];
    }

    return (
      <div className={ styles.buttonbar }>
        { buttons }
      </div>
    );
  }

  onCancelClick = () => {
    if (this.store.isEditing) {
      this.store.setEditing(false);
    } else if (this.store.isNew) {
      this.store.setNew(false);
    }
  }

  onDeleteClick = () => {
    if (!this.store.currentApp.isOwner && !this.store.isContractOwner) {
      return;
    }
  }

  onEditClick = () => {
    if (!this.store.currentApp.isOwner) {
      return;
    }

    this.store.setEditing(true);
  }

  onNewClick = () => {
    this.store.setNew(true);
  }

  onSaveClick = () => {
    if (!this.store.canSave) {
      return;
    }
  }
}
