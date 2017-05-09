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

import { observer } from 'mobx-react';
import React, { Component } from 'react';

import EvalStore from '../evalStore';

import styles from './autocomplete.css';

@observer
export default class Autocomplete extends Component {
  evalStore = EvalStore.get();

  render () {
    if (!this.evalStore.showAutocomplete) {
      return null;
    }

    return (
      <div className={ styles.container }>
        { this.renderAutocompletes() }
      </div>
    );
  }

  renderAutocompletes () {
    const { autocompletes } = this.evalStore;
    const displayedProto = {};

    return autocompletes.map((autocomplete, index) => {
      const { name, prototypeName } = autocomplete;
      const onClick = () => this.handleClick(autocomplete);

      const proto = !displayedProto[prototypeName]
        ? (
          <span className={ styles.proto }>
            { prototypeName }
          </span>
        )
        : null;

      if (!displayedProto[prototypeName]) {
        displayedProto[prototypeName] = true;
      }

      return (
        <div
          className={ styles.item }
          key={ index }
          onClick={ onClick }
        >
          <span>
            { name }
          </span>
          { proto }
        </div>
      );
    });
  }

  handleClick = (autocomplete) => {
    this.evalStore.selectAutocomplete(autocomplete);
  }
}
