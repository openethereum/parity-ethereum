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

import EvalStore from '../evalStore';

import styles from './input.css';

@observer
export default class Input extends Component {
  evalStore = EvalStore.get();

  render () {
    const { input } = this.evalStore;

    return (
      <div className={ styles.container }>
        <span className={ styles.type }>&gt;</span>
        <input
          className={ styles.input }
          onChange={ this.handleChange }
          onKeyUp={ this.handleKeyUp }
          type='text'
          value={ input }
        />
      </div>
    );
  }

  handleChange = (event) => {
    const { value } = event.target;

    this.evalStore.updateInput(value);
  }

  handleKeyUp = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      this.evalStore.evaluate();
    }
  }
}
