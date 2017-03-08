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
import DappsStore from '../dappsStore';
import Button from '../Button';
import Input from '../Input';
import ModalDelete from '../ModalDelete';
import ModalUpdate from '../ModalUpdate';
import SelectAccount from '../SelectAccount';

import styles from './dappModal.css';

@observer
export default class DappModal extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired,
    open: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  };

  state = {
    showDelete: false,
    showUpdate: false,
    updates: null
  };

  dappsStore = DappsStore.instance();

  componentWillReceiveProps (nextProps) {
    if (nextProps.open && !this.props.open) {
      this.handleOpen();
    }
  }

  render () {
    const { dapp, open } = this.props;
    const { showDelete, showUpdate, updates } = this.state;

    const classes = [ styles.modal ];

    if (open) {
      classes.push(styles.open);
    }

    return (
      <div
        className={ classes.join(' ') }
        onClick={ open && !(showDelete || showUpdate) ? this.handleClose : null }
        onKeyUp={ this.handleKeyPress }
      >
        {
          showDelete
          ? (
            <ModalDelete
              dappId={ dapp.id }
              onClose={ this.handleDeleteClose }
              onDelete={ this.handleDeleteConfirm }
            />
          )
          : null
        }
        {
          showUpdate
          ? (
            <ModalUpdate
              dappId={ dapp.id }
              onClose={ this.handleUpdateClose }
              onConfirm={ this.handleUpdateConfirm }
              updates={ updates }
            />
          )
          : null
        }
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
    const manifest = dapp.manifest.content || {};

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

    const { isEditing } = dapp;

    if (isEditing) {
      return (
        <div className={ styles.actions }>
          <Button
            className={ styles.button }
            disabled={ !dapp.canSave }
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
    const { id, image } = dapp;
    const manifest = dapp.manifest.content || {};

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
          <img src={ image.url } />
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

  renderInputs (dapp) {
    return [
      this.renderOwner(dapp),
      this.renderHashInput(dapp, 'image', 'Image URL', true),
      this.renderHashInput(dapp, 'manifest', 'Manifest URL'),
      this.renderHashInput(dapp, 'content', 'Content URL')
    ];
  }

  renderOwner (dapp) {
    const { isEditing } = dapp;

    if (isEditing) {
      return this.renderOwnerSelect(dapp);
    }

    return this.renderOwnerStatic(dapp);
  }

  renderOwnerSelect (dapp) {
    const overlayImage = (
      <img
        className={ styles.overlayImage }
        src={ api.util.createIdentityImg(this.props.dapp.wip.owner.address, 4) }
      />
    );

    return (
      <Input
        key='owner_select'
        hint={ this.props.dapp.wip.owner.address }
        label='Owner, select the application owner and editor'
        overlay={ overlayImage }
      >
        <SelectAccount
          onSelect={ this.handleSelectOwner }
          value={ dapp.wip.owner.address }
        />
      </Input>
    );
  }

  renderOwnerStatic (dapp) {
    const overlayImage = (
      <img
        className={ styles.overlayImage }
        src={ api.util.createIdentityImg(dapp.owner.address, 4) }
      />
    );

    return (
      <Input
        key='owner_static'
        hint={ dapp.owner.address }
        label='Owner, the application owner and editor'
        overlay={ overlayImage }
      >
        <input
          readOnly
          tabIndex={ -1 }
          value={ dapp.owner.name || dapp.owner.address }
        />
      </Input>
    );
  }

  renderHashInput (dapp, type, label, isImage = false) {
    const handleChange = (event) => {
      return this.handleChangeHash(event, type);
    };

    const { isEditing, wip } = dapp;

    const changed = wip && wip[type].changed;
    const error = wip && wip[type].error;

    const hash = dapp[type].hash;
    const url = dapp[type].url;

    const overlayImage = (isImage && hash)
      ? (
        <img
          className={ styles.overlayImage }
          src={ `/api/content/${hash.substr(2)}` }
        />
      )
      : null;

    const wipUrl = isEditing && wip && wip[type].url;

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
          tabIndex={ isEditing ? 0 : -1 }
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

    const changed = (this.props.dapp.owner.address !== value);

    this.props.dapp.handleChange({
      owner: {
        address: value,
        changed
      }
    });
  }

  handleChangeHash = (event, type) => {
    if (!this.props.dapp.isEditing) {
      return;
    }

    const url = event.target.value;

    let changed = (this.props.dapp[type].url !== url);

    this.props.dapp.handleChange({
      [ type ]: {
        error: null,
        changed,
        url
      }
    });
  }

  handleCancel = () => {
    this.props.dapp.setEditing(false);
  }

  handleEdit = () => {
    this.props.dapp.setEditing(true);
  }

  handleDelete = () => {
    this.setState({ showDelete: true });
  }

  handleDeleteClose = () => {
    this.setState({ showDelete: false });
  }

  handleDeleteConfirm = () => {
    const { id, owner } = this.props.dapp;

    this.dappsStore.delete(id, owner.address);
    this.handleDeleteClose();
    this.handleClose();
  }

  handleSave = () => {
    const updates = this.props.dapp.handleSave();

    this.setState({ showUpdate: true, updates });
  }

  handleUpdateClose = () => {
    this.setState({ showUpdate: false, updates: null });
  }

  handleUpdateConfirm = () => {
    const { id, owner } = this.props.dapp;
    const { updates } = this.state;

    this.dappsStore.update(id, owner.address, updates);
    this.handleUpdateClose();
  }
}
