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
import { action, computed, map, observable, transaction } from 'mobx';
import store from 'store';

import { evaluate } from '../utils';

const LS_SNIPPETS_KEY = '_console::snippets';

let instance;

export default class SnippetsStore {
  @observable files = map();
  @observable nextName = null;
  @observable renaming = null;
  @observable selected = null;

  codeMirror = null;

  constructor () {
    this.load();
  }

  static get () {
    if (!instance) {
      instance = new SnippetsStore();
    }

    return instance;
  }

  @computed
  get code () {
    if (!this.selected) {
      return '';
    }

    return this.files.get(this.selected).content;
  }

  @action
  cancelRename () {
    if (!this.renaming || !this.nextName) {
      return;
    }

    this.renaming = null;
    this.nextName = null;
  }

  clearCodeHistory () {
    if (this.codeMirror) {
      this.codeMirror.doc.clearHistory();
    }
  }

  @action
  create () {
    const id = this.getNewId();
    const file = {
      content: '',
      isPristine: false,
      name: `Snippet #${id}`,
      id
    };

    transaction(() => {
      this.files.set(id, file);
      this.select(id);
    });
  }

  edit (value) {
    if (!this.selected) {
      this.create();
    }

    const file = this.files.get(this.selected);

    file.content = value;
    this.updateFile(file);
  }

  evaluate () {
    const code = this.code;

    if (!code) {
      return;
    }

    const { result, error } = evaluate(code);

    if (error) {
      console.error(error);
    } else {
      console.log(result);
    }
  }

  getFromStorage () {
    return store.get(LS_SNIPPETS_KEY) || [];
  }

  getNewId () {
    if (this.files.size === 0) {
      return 1;
    }

    const ids = this.files.values().map((file) => file.id);

    return Math.max(...ids) + 1;
  }

  load () {
    const files = this.getFromStorage();

    transaction(() => {
      files.forEach((file) => {
        this.files.set(file.id, file);
      });
    });
  }

  @action
  remove (id) {
    transaction(() => {
      if (id === this.selected) {
        this.selected = null;
      }

      this.files.delete(id);

      const files = this.getFromStorage()
        .filter((f) => f.id !== id);

      return store.set(LS_SNIPPETS_KEY, files);
    });
  }

  save (_file) {
    let file;

    if (!_file) {
      if (!this.selected) {
        return false;
      }

      file = this.files.get(this.selected);
    } else {
      file = _file;
    }

    file.savedContent = file.content;
    this.updateFile(file);
    this.saveToStorage(file);
  }

  saveName () {
    if (!this.renaming || !this.nextName) {
      return;
    }

    const file = this.files.get(this.renaming);

    file.name = this.nextName;

    this.save(file);
    this.cancelRename();
  }

  saveToStorage (file) {
    const files = this.getFromStorage();
    const index = files.findIndex((f) => file.id === f.id);

    if (index === -1) {
      files.push(file);
    } else {
      files[index] = file;
    }

    return store.set(LS_SNIPPETS_KEY, files);
  }

  @action
  select (id) {
    this.selected = id;

    // Wait for the file content to be loaded
    setTimeout(() => {
      this.clearCodeHistory();
    }, 50);
  }

  setCodeMirror (codeMirror) {
    this.codeMirror = codeMirror;

    if (!codeMirror) {
      return;
    }

    this.codeMirror
      .on('keydown', (_, event) => {
        const codeName = keycode(event);

        if (codeName === 'enter' && event.ctrlKey) {
          event.preventDefault();
          event.stopPropagation();
          return this.evaluate();
        }
      });
  }

  @action
  startRename (id) {
    const file = this.files.get(id);

    transaction(() => {
      this.renaming = id;
      this.nextName = file.name;
    });
  }

  @action
  updateFile (file) {
    file.isPristine = (file.content === file.savedContent);

    this.files.set(file.id, file);
  }

  @action
  updateName (value) {
    this.nextName = value;
  }
}
