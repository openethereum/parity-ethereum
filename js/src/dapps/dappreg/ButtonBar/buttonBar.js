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

import Button from '../Button';
import Store from '../store';
import styles from './ButtonBar.css';

@observer
export default class ButtonBar extends Component {
  store = Store.instance();

  render () {
    let buttons = [];

    if (this.store.isEditing || this.store.isNew) {
      buttons = [
        <Button
          key='cancel'
          label='Cancel'
          className={ styles.cancel }
          onClick={ this.onCancelClick } />,
        <Button
          key='save'
          label={ this.store.isNew ? 'Register' : 'Update' }
          disabled={ !this.store.canSave }
          onClick={ this.onSaveClick } />
      ];
    } else {
      buttons = [
        <Button
          key='delete'
          label='Delete'
          className={ styles.delete }
          disabled={ !this.store.currentApp.isOwner && !this.store.isContractOwner }
          onClick={ this.onDeleteClick } />,
        <Button
          key='edit'
          label='Edit'
          disabled={ !this.store.currentApp.isOwner }
          onClick={ this.onEditClick } />,
        <Button
          key='new'
          label='New'
          onClick={ this.onNewClick } />
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
  }

  onEditClick = () => {
    this.store.setEditing(true);
  }

  onNewClick = () => {
    this.store.setNew(true);
  }

  onSaveClick = () => {
  }
}
