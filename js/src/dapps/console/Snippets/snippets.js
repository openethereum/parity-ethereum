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
              lineNumbers: true,
              mode: 'javascript'
            } }
            value={ code }
          />
        </div>
      </div>
    );
  }

  renderFiles () {
    const { files, selected } = this.snippetsStore;

    return files.values().map((file) => {
      const { id, name } = file;
      const classes = [ styles.file ];
      const onClick = () => this.handleSelectFile(id);

      if (selected === id) {
        classes.push(styles.selected);
      }

      return (
        <div
          className={ classes.join(' ') }
          key={ id }
          onClick={ onClick }
        >
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
    });
  }

  handleAddFile = () => {
    this.snippetsStore.create();
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

  handleSelectFile = (id) => {
    this.snippetsStore.select(id);
  };

  setRef = (node) => {
    const codeMirror = node
      ? node.getCodeMirror()
      : null;

    this.snippetsStore.setCodeMirror(codeMirror);
  };
}
