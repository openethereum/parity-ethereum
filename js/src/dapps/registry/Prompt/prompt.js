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

import EventListener from 'react-event-listener';
import keycode from 'keycode';
import { observer } from 'mobx-react';
import React, { Component } from 'react';
import RaisedButton from 'material-ui/RaisedButton';

import PromptStore from './prompt.store';

import styles from './prompt.css';

@observer
export default class Prompt extends Component {
  promptStore = PromptStore.get();

  render () {
    const { show } = this.promptStore;

    if (!show) {
      return null;
    }

    const { showInput, title } = this.promptStore;

    return (
      <div className={ styles.prompt }>
        <EventListener
          onKeyUp={ this.handleWindowKeyUp }
          target='window'
        />
        <div className={ styles.container }>
          <div className={ styles.label }>
            { title }
          </div>
          { this.renderInput() }
          <div className={ styles.actions }>
            <RaisedButton
              label='Close'
              onClick={ this.handleClose }
            />
            <RaisedButton
              autoFocus={ !showInput }
              label='Submit'
              onClick={ this.handleSubmit }
              primary
            />
          </div>
        </div>
      </div>
    );
  }

  renderInput () {
    if (!this.promptStore.showInput) {
      return null;
    }

    const { defaultValue, placeholder } = this.promptStore;

    return (
      <div className={ styles.input }>
        <input
          autoFocus
          defaultValue={ defaultValue }
          onChange={ this.handleChange }
          onKeyUp={ this.handleInputKeyUp }
          placeholder={ placeholder }
          type='text'
        />
      </div>
    );
  }

  handleChange = (event) => {
    const { value } = event.target;

    this.promptStore.updateInput(value);
  };

  handleClose = () => {
    this.promptStore.close();
  };

  handleInputKeyUp = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      this.promptStore.submit();
    }
  };

  handleSubmit = () => {
    this.promptStore.submit();
  };

  handleWindowKeyUp = (event) => {
    const codeName = keycode(event);

    if (codeName === 'esc') {
      this.promptStore.close();
    }
  };
}
