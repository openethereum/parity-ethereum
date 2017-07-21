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
import React, { Component } from 'react';
import CodeMirror from 'react-codemirror';
import EventListener from 'react-event-listener';

import Console from '../Console';
import SnippetsStore from './snippets.store';

import styles from './snippets.css';

@observer
export default class Snippets extends Component {
  snippetsStore = SnippetsStore.get();

  render () {
    const { code } = this.snippetsStore;

    return (
      <div className={ styles.container }>
        <EventListener
          onKeyDown={ this.handleKeyDown }
          target='window'
        />
        <div className={ styles.panel }>
          <div
            className={ styles.add }
            onClick={ this.handleAddFile }
          >
            <span className={ styles.plus }>+</span>
            <span>New Snippet</span>
          </div>
          <div className={ styles.list }>
            { this.renderFiles() }
          </div>
        </div>
        <div className={ styles.code }>
          <CodeMirror
            ref={ this.setRef }
            onChange={ this.handleChange }
            options={ {
              autofocus: true,
              extraKeys: {
                'Ctrl-Space': 'autocomplete'
              },
              keyMap: 'sublime',
              highlightSelectionMatches: {
                delay: 0,
                showToken: false
              },
              lineNumbers: true,
              mode: 'javascript'
            } }
            value={ code }
          />
          <div className={ styles.console }>
            <Console />
          </div>
        </div>
      </div>
    );
  }

  renderFiles () {
    const { files } = this.snippetsStore;

    return files
      .values()
      .sort((fa, fb) => fa.name.localeCompare(fb.name))
      .map((file) => this.renderFile(file));
  }

  renderFile (file) {
    const { nextName, renaming, selected } = this.snippetsStore;
    const { id, name } = file;
    const classes = [ styles.file ];

    if (renaming === id) {
      return (
        <div
          className={ classes.join(' ') }
          key={ id }
        >
          <EventListener
            onClick={ this.handleSaveName }
            target='window'
          />
          <div className={ styles.inputContainer }>
            <input
              autoFocus
              className={ styles.input }
              onClick={ this.stopPropagation }
              onChange={ this.handleNameChange }
              onKeyDown={ this.handleRenameKeyDown }
              type='text'
              value={ nextName }
            />
          </div>
        </div>
      );
    }

    const onClick = () => this.handleSelectFile(id);
    const onDoubleClick = () => this.handleRenameFile(id);
    const onRemove = (event) => this.handleRemove(id, event);

    if (selected === id) {
      classes.push(styles.selected);
    }

    return (
      <div
        className={ classes.join(' ') }
        key={ id }
        onClick={ onClick }
        onDoubleClick={ onDoubleClick }
      >
        <span
          className={ styles.remove }
          onClick={ onRemove }
          title={ `Remove ${name}` }
        >
          ✖
        </span>
        <span className={ styles.pristine }>
          {
            file.isPristine
            ? null
            : '*'
          }
        </span>
        <span>
          { name }
        </span>
      </div>
    );
  }

  handleAddFile = () => {
    this.snippetsStore.create();
  };

  handleSaveName = (event) => {
    this.snippetsStore.saveName();
    return event;
  };

  handleChange = (value) => {
    this.snippetsStore.edit(value);
  };

  handleKeyDown = (event) => {
    const codeName = keycode(event);

    if (codeName === 's' && event.ctrlKey) {
      event.preventDefault();
      event.stopPropagation();

      return this.snippetsStore.save();
    }
  };

  handleNameChange = (event) => {
    const { value } = event.target;

    this.snippetsStore.updateName(value);
  };

  handleRemove = (id, event) => {
    this.snippetsStore.remove(id);
    event.stopPropagation();
  };

  handleRenameFile = (id) => {
    this.snippetsStore.startRename(id);
  };

  handleRenameKeyDown = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      return this.snippetsStore.saveName();
    }

    if (codeName === 'esc') {
      return this.snippetsStore.cancelRename();
    }
  };

  handleSelectFile = (id) => {
    this.snippetsStore.select(id);
  };

  setRef = (node) => {
    const codeMirror = node
      ? node.getCodeMirror()
      : null;

    this.snippetsStore.setCodeMirror(codeMirror);
  };

  stopPropagation = (event) => {
    event.stopPropagation();
  };
}
