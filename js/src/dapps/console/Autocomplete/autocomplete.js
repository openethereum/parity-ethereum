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
import ReactDOM from 'react-dom';

import AutocompleteStore from './autocomplete.store';

import styles from './autocomplete.css';

@observer
export default class Autocomplete extends Component {
  autocompleteStore = AutocompleteStore.get();

  render () {
    if (!this.autocompleteStore.show) {
      return null;
    }

    return (
      <div
        className={ styles.container }
        style={ this.autocompleteStore.position }
      >
        { this.renderAutocompletes() }
      </div>
    );
  }

  renderAutocompletes () {
    const { selected, values } = this.autocompleteStore;
    const displayedProto = {};

    return values.map((autocomplete, index) => {
      const { name, prototypeName } = autocomplete;
      const onClick = () => this.handleClick(index);
      const setRef = (node) => this.setRef(index, node);

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

      const classes = [ styles.item ];

      if (index === selected) {
        classes.push(styles.selected);
      }

      return (
        <div
          className={ classes.join(' ') }
          key={ index }
          onClick={ onClick }
          ref={ setRef }
        >
          <span>
            { name }
          </span>
          { proto }
        </div>
      );
    });
  }

  handleClick = (index) => {
    this.autocompleteStore.select(index);
  };

  setRef = (index, node) => {
    const element = ReactDOM.findDOMNode(node);

    this.autocompleteStore.setElement(index, element);
  };
}
