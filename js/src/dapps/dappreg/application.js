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

import Store from './store';
import styles from './application.css';

import { api } from './parity';

@observer
export default class Application extends Component {
  store = Store.instance();

  state = {
  }

  render () {
    return this.store.isLoading
      ? this.renderLoading()
      : this.renderApplication();
  }

  renderApplication () {
    return (
      <div className={ styles.body }>
        { this.renderWarning() }
        <div className={ styles.apps }>
          { this.renderAppsSelect() }
          { this.renderButtons() }
          { this.renderCurrentApp() }
        </div>
        { this.renderFooter() }
      </div>
    );
  }

  renderAppsSelect () {
    if (this.store.isNew) {
      return null;
    }

    const overlayImg = this.store.currentApp.imageHash
      ? <img
        className={ styles.overlayImage }
        src={ `/api/content/${this.store.currentApp.imageHash.substr(2)}` } />
      : null;

    return (
      <div className={ styles.overlayContainer }>
        <label>Application, the actual application details to show below</label>
        <select
          value={ this.store.currentApp.id }
          disabled={ this.store.isEditing }
          onChange={ this.onSelectApp }>
          {
            this.store.apps.map((app) => {
              return (
                <option value={ app.id } key={ app.id }>
                  { app.name }
                </option>
              );
            })
          }
        </select>
        <div className={ styles.hint }>{ this.store.currentApp.id }</div>
        { overlayImg }
      </div>
    );
  }

  cannotSave () {
    const app = this.state;
    const hasError = app.contentError || app.imageError || app.manifestError;
    const isDirty = this.store.isNew || app.contentChanged || app.imageChanged || app.manifestChanged;
    const isEdited = this.store.isEditing || this.store.isNew;

    return !isEdited || hasError || !isDirty;
  }

  renderButtons () {
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
          disabled={ this.cannotSave() }
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

  renderCurrentApp () {
    const app = this.store.isNew || this.store.isEditing
      ? this.state
      : this.store.currentApp;
    const ownerLabel = <label>Owner, the application owner and editor</label>;
    const overlayImg = app.imageHash
      ? <img
        className={ styles.overlayImage }
        src={ `/api/content/${app.imageHash.substr(2)}` } />
      : null;

    let ownerInput;
    if (this.store.isNew) {
      ownerInput = (
        <div className={ styles.overlayContainer }>
          { ownerLabel }
          <select
            value={ this.store.currentAccount.address }
            onChange={ this.onSelectAccount }>
            {
              this.store.accounts.map((account) => {
                return (
                  <option value={ account.address } key={ account.address }>
                    { account.name }
                  </option>
                );
              })
            }
          </select>
          <div className={ styles.hint }>{ app.owner }</div>
          <img
            className={ styles.overlayImage }
            src={ api.util.createIdentityImg(this.store.currentAccount.address, 4) } />
        </div>
      );
    } else {
      ownerInput = (
        <div className={ styles.overlayContainer }>
          { ownerLabel }
          <input value={ app.ownerName } readOnly />
          <div className={ styles.hint }>{ app.owner }</div>
          <img
            className={ styles.overlayImage }
            src={ api.util.createIdentityImg(app.owner, 4) } />
        </div>
      );
    }

    return (
      <div className={ styles.app }>
        <div className={ styles.section }>
          <div>
            <label>Application Id, the unique assigned identifier</label>
            <input value={ app.id } readOnly />
            <div className={ styles.hint }>...</div>
          </div>
          { ownerInput }
        </div>
        <div className={ styles.section }>
          <div className={ styles.overlayContainer }>
            <label>Image hash, as generated by Githubhint</label>
            <input value={ app.imageHash || '' } data-dirty={ app.imageChanged } readOnly={ !this.store.isEditing && !this.store.isNew } data-error={ !!app.imageError } onChange={ this.onChangeImage } />
            <div className={ styles.hint }>{ app.imageError || app.imageUrl || '...' }</div>
            { overlayImg }
          </div>
          <div>
            <label>Manifest hash, as generated by Githubhint</label>
            <input value={ app.manifestHash || '' } data-dirty={ app.manifestChanged } readOnly={ !this.store.isEditing && !this.store.isNew } data-error={ !!app.manifestError } onChange={ this.onChangeManifest } />
            <div className={ styles.hint }>{ app.manifestError || app.manifestUrl || '...' }</div>
          </div>
          <div>
            <label>Content hash, as generated by Githubhint</label>
            <input value={ app.contentHash || '' } data-dirty={ app.contentChanged } readOnly={ !this.store.isEditing && !this.store.isNew } data-error={ !!app.contentError } onChange={ this.onChangeContent } />
            <div className={ styles.hint }>{ app.contentError || app.contentUrl || '...' }</div>
          </div>
        </div>
      </div>
    );
  }

  renderFooter () {
    return (
      <div className={ styles.footer }>
        { this.store.count } applications registered, { this.store.ownedCount } owned by user
      </div>
    );
  }

  renderLoading () {
    return (
      <div className={ styles.body }>
        <div className={ styles.loading }>Loading application</div>
      </div>
    );
  }

  renderWarning () {
    return (
      <div className={ styles.warning }>
        WARNING: Registering a dapp is for developers only. Please ensure you understand the steps needed to develop and deploy applications, should you wish to use this dapp for anything apart from queries. A non-refundable fee of { api.util.fromWei(this.store.fee).toFormat(3) }<small>ETH</small> is required for any registration.
      </div>
    );
  }

  copyToState () {
    const app = this.store.currentApp;

    this.setState({
      id: this.store.isNew ? this.store.newId : app.id,
      contentChanged: false,
      contentError: null,
      contentHash: this.store.isNew ? null : app.contentHash,
      contentUrl: this.store.isNew ? null : app.contentUrl,
      imageChanged: false,
      imageError: null,
      imageHash: this.store.isNew ? null : app.imageHash,
      imageUrl: this.store.isNew ? null : app.imageUrl,
      manifestChanged: false,
      manifestError: null,
      manifestHash: this.store.isNew ? null : app.manifestHash,
      manifestUrl: this.store.isNew ? null : app.manifestUrl,
      owner: this.store.isNew ? this.store.currentAccount.address : app.owner,
      ownerName: this.store.isNew ? this.store.currentAccount.name : app.ownerName
    });
  }

  onSelectAccount = (event) => {
    this.store.setCurrentAccount(event.target.value);
  }

  onSelectApp = (event) => {
    this.store.setCurrentApp(event.target.value);
  }

  onChangeHash (event, type) {
    if (!this.store.isNew && !this.store.isEditing) {
      return;
    }

    const hash = event.target.value;
    let changed = false;
    let url = null;

    if (this.store.isNew) {
      if (hash && hash.length) {
        changed = true;
      }
    } else {
      if (this.store.currentApp[`${type}Hash`] !== hash) {
        changed = true;
      } else {
        url = this.store.currentApp[`${type}Url`];
      }
    }

    this.setState({
      [`${type}Changed`]: changed,
      [`${type}Error`]: null,
      [`${type}Hash`]: hash,
      [`${type}Url`]: changed ? 'Resolving url from hash' : url
    }, () => {
      if (changed) {
        this.store
          .lookupHash(hash)
          .then((url) => {
            this.setState({
              [`${type}Error`]: url ? null : 'Unable to resolve url',
              [`${type}Url`]: url
            });
          });
      }
    });
  }

  onChangeContent = (event) => {
    this.onChangeHash(event, 'content');
  }

  onChangeImage = (event) => {
    this.onChangeHash(event, 'image');
  }

  onChangeManifest = (event) => {
    this.onChangeHash(event, 'manifest');
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
    this.copyToState();
  }

  onNewClick = () => {
    this.store.setNew(true);
    this.copyToState();
  }

  onSaveClick = () => {
    if (this.cannotSave()) {
      return;
    }
  }
}
