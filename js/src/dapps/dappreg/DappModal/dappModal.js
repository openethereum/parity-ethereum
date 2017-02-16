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

import keycode from 'keycode';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';

import { api } from '../parity';
import Button from '../Button';
import DappsStore from '../dappsStore';
import ModalStore from '../modalStore';
import Input from '../Input';
import SelectAccount from '../SelectAccount';

import styles from './dappModal.css';

@observer
export default class DappCard extends Component {
  dappsStore = DappsStore.instance();
  modalStore = ModalStore.instance();

  static propTypes = {
    dapp: PropTypes.object.isRequired,
    open: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  };

  componentWillReceiveProps (nextProps) {
    if (nextProps.open && !this.props.open) {
      this.handleOpen();
    }
  }

  render () {
    const { dapp, open } = this.props;

    const classes = [ styles.modal ];

    if (open) {
      classes.push(styles.open);
    }

    return (
      <div
        className={ classes.join(' ') }
        onClick={ open ? this.handleClose : null }
        onKeyUp={ this.handleKeyPress }
      >
        <div
          className={ styles.container }
          onClick={ open ? this.stopEvent : null }
          ref='container'
          tabIndex={ open ? 0 : null }
        >
          <div
            className={ styles.close }
            onClick={ this.handleClose }
            onKeyPress={ this.handleCloseKeyPress }
            tabIndex={ open ? 0 : null }
            title='close'
          >
            ❌
          </div>

          { this.renderHeader(dapp) }
          { this.renderContent(dapp) }
        </div>
      </div>
    );
  }

  renderContent (dapp) {
    const manifest = dapp.manifest || {};

    return (
      <div className={ styles.content }>
        <div>
          { this.renderInputs(dapp) }
        </div>

        { this.renderActions(dapp) }

        <div className={ styles.code }>
          <div className={ styles.codeTitle }>manifest.json</div>
          <div className={ styles.codeContainer }>
            <code>{ JSON.stringify(manifest, null, 2) }</code>
          </div>
        </div>
      </div>
    );
  }

  renderActions (dapp) {
    if (!dapp.isOwner) {
      return null;
    }

    const { isEditing } = this.dappsStore;

    if (isEditing) {
      return (
        <div className={ styles.actions }>
          <Button
            className={ styles.button }
            disabled={ !this.dappsStore.canSave }
            label='Save'
            onClick={ this.handleSave }
          />
          <Button
            className={ styles.button }
            label='Cancel'
            onClick={ this.handleCancel }
            warning
          />
        </div>
      );
    }

    return (
      <div className={ styles.actions }>
        <Button
          className={ styles.button }
          label='Edit'
          onClick={ this.handleEdit }
        />
        <Button
          className={ styles.button }
          label='Delete'
          onClick={ this.handleDelete }
          warning
        />
      </div>
    );
  }

  renderHeader (dapp) {
    const { id, imageUrl } = dapp;
    const manifest = dapp.manifest || {};

    const infos = [];

    if (manifest.version) {
      infos.push(`v${manifest.version}`);
    }

    if (manifest.author) {
      infos.push(`by ${manifest.author}`);
    }

    return (
      <div className={ styles.header }>
        <div className={ styles.icon }>
          <img src={ imageUrl } />
        </div>
        <div>
          <div className={ styles.name }>
            { manifest.name || 'Unnamed' }
          </div>
          <div className={ styles.info }>
            { id }
          </div>
          <div className={ styles.info }>
            { infos.length > 0 ? infos.join(', ') : null }
          </div>
        </div>
      </div>
    );
  }

  renderInputs (app) {
    return [
      this.renderOwner(app),
      this.renderHashInput(app, 'image', 'Image URL', true),
      this.renderHashInput(app, 'manifest', 'Manifest URL'),
      this.renderHashInput(app, 'content', 'Content URL')
    ];
  }

  renderOwner (app) {
    const { isEditing } = this.dappsStore;

    if (isEditing) {
      return this.renderOwnerSelect(app);
    }

    return this.renderOwnerStatic(app);
  }

  renderOwnerSelect (app) {
    const overlayImage = (
      <img
        className={ styles.overlayImage }
        src={ api.util.createIdentityImg(this.dappsStore.currentAccount.address, 4) }
      />
    );

    return (
      <Input
        key='owner_select'
        hint={ this.dappsStore.currentAccount.address }
        label='Owner, select the application owner and editor'
        overlay={ overlayImage }
      >
        <SelectAccount
          onSelect={ this.handleSelectOwner }
        />
      </Input>
    );
  }

  renderOwnerStatic (app) {
    const overlayImage = (
      <img
        className={ styles.overlayImage }
        src={ api.util.createIdentityImg(app.owner, 4) }
      />
    );

    return (
      <Input
        key='owner_static'
        hint={ app.owner }
        label='Owner, the application owner and editor'
        overlay={ overlayImage }
      >
        <input value={ app.ownerName } readOnly />
      </Input>
    );
  }

  renderHashInput (app, type, label, isImage = false) {
    const handleChange = (event) => {
      return this.handleChangeHash(event, type);
    };

    const { isEditing, wipApp } = this.dappsStore;

    const changed = wipApp && wipApp[`${type}Changed`];
    const error = app[`${type}Error`];
    const hash = app[`${type}Hash`];
    const url = app[`${type}Url`];

    const overlayImage = (isImage && hash)
      ? (
        <img
          className={ styles.overlayImage }
          src={ `/api/content/${hash.substr(2)}` }
        />
      )
      : null;

    const wipUrl = isEditing && wipApp && wipApp[`${type}Url`];

    const hint = error || (!changed && hash) || '...';
    const value = wipUrl || url || '';

    return (
      <Input
        key={ `${type}Edit` }
        hint={ hint }
        label={ label }
        overlay={ overlayImage }
      >
        <input
          data-dirty={ changed }
          data-error={ !!error }
          onChange={ handleChange }
          readOnly={ !isEditing }
          value={ value }
        />
      </Input>
    );
  }

  stopEvent = (event) => {
    event.stopPropagation();
    event.preventDefault();

    return false;
  }

  handleKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'esc') {
      return this.handleClose();
    }

    return event;
  }

  handleCloseKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      return this.handleClose();
    }

    return event;
  }

  handleOpen = () => {
    if (!this.refs.container) {
      return false;
    }

    // Focus after the modal is open
    setTimeout(() => {
      const element = ReactDOM.findDOMNode(this.refs.container);

      element && element.focus();
    }, 50);
  }

  handleClose = () => {
    this.handleCancel();
    this.props.onClose();
  }

  handleSelectOwner = (event) => {
    const { value } = event.target;

    const changed = (this.dappsStore.currentApp.owner !== value);

    this.dappsStore.editWip({
      ownerChanged: changed,
      owner: value
    });
  }

  handleChangeHash = (event, type) => {
    if (!this.dappsStore.isEditing) {
      return;
    }

    const url = event.target.value;

    let changed = (this.dappsStore.currentApp[`${type}Url`] !== url);

    this.dappsStore.editWip({
      [`${type}Changed`]: changed,
      [`${type}Error`]: null,
      [`${type}Url`]: url
    });
  }

  handleCancel = () => {
    this.dappsStore.setEditing(false);
  }

  handleDelete = () => {
    this.modalStore.showDelete();
  }

  handleEdit = () => {
    this.dappsStore.setEditing(true);
  }

  handleSave = () => {
    this.modalStore.showUpdate();
  }
}
