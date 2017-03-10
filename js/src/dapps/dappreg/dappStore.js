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

import { action, computed, observable, transaction } from 'mobx';

export default class DappStore {
  @observable id = null;
  @observable content = null;
  @observable image = null;
  @observable manifest = null;
  @observable owner = null;
  @observable isOwner = false;

  @observable isEditing = false;
  @observable wip = null;

  contractOwner = '';
  isContractOwner = false;

  constructor (data) {
    const { id, content = {}, image = {}, manifest = {}, owner = {}, isOwner = false, contractOwner = '', isContractOwner = false } = data;

    transaction(() => {
      this.id = id;
      this.content = content;
      this.image = image;
      this.manifest = manifest;
      this.owner = owner;
      this.isOwner = isOwner;

      this.copyToWip();
    });

    this.contractOwner = contractOwner;
    this.isContractOwner = isContractOwner;
  }

  @computed get canSave () {
    if (!this.wip) {
      return false;
    }

    const { content, image, manifest, owner } = this.wip;
    const fields = [ content, image, manifest, owner ];

    const hasError = !!fields.find((field) => field.error);
    const hasChanged = !!fields.find((field) => field.changed);
    const isEditMode = this.isEditing;

    return isEditMode && hasChanged && !hasError;
  }

  @action copyToWip = () => {
    const defaults = {
      changed: false,
      error: null
    };

    const wip = {
      id: this.id,
      content: {
        ...defaults,
        url: this.content.url
      },
      image: {
        ...defaults,
        url: this.image.url
      },
      manifest: {
        ...defaults,
        url: this.manifest.url
      },
      owner: {
        ...defaults,
        address: this.owner.address
      }
    };

    this.wip = { ...wip };
  }

  @action handleChange = (details) => {
    if (!this.isEditing) {
      return false;
    }

    this.wip = {
      ...this.wip,
      ...details
    };

    return this.wip;
  }

  @action handleSave = () => {
    const updates = {};

    if (this.wip.content.url !== this.content.url) {
      updates.content = this.wip.content.url;
    }

    if (this.wip.image.url !== this.image.url) {
      updates.image = this.wip.image.url;
    }

    if (this.wip.manifest.url !== this.manifest.url) {
      updates.manifest = this.wip.manifest.url;
    }

    if (this.wip.owner.address !== this.owner.address) {
      updates.owner = this.wip.owner.address;
    }

    return updates;
  }

  @action setEditing = (mode) => {
    transaction(() => {
      this.isEditing = mode;
      this.copyToWip();
    });

    return mode;
  }

  update = (updates) => {
    const { image, content } = updates;
    const changes = {};

    if (image && image !== this.wip.image.url) {
      changes.image = { url: image, changed: true };
    }

    if (content && content !== this.wip.content.url) {
      changes.content = { url: content, changed: true };
    }

    return this.handleChange(changes);
  }
}
