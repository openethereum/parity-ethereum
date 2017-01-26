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

import React, { Component } from 'react';

import PlaygroundExample from '~/playground/playgroundExample';
import SectionList from './';

const ITEM_STYLE = {
  backgroundColor: 'rgba(0, 0, 0, 0.75)',
  padding: '1em'
};

const items = [
  { name: 'Jack', desc: 'Item number 1' },
  { name: 'Paul', desc: 'Item number 2' },
  { name: 'Matt', desc: 'Item number 3' },
  { name: 'Titi', desc: 'Item number 4' }
];

export default class SectionListExample extends Component {
  state = {
    showOverlay: true
  };

  render () {
    return (
      <div>
        <PlaygroundExample name='Simple Usage'>
          { this.renderSimple() }
        </PlaygroundExample>

        <PlaygroundExample name='With Overlay'>
          { this.renderWithOverlay() }
        </PlaygroundExample>
      </div>
    );
  }

  renderSimple () {
    return (
      <SectionList
        items={ items }
        renderItem={ this.renderItem }
      />
    );
  }

  renderWithOverlay () {
    const { showOverlay } = this.state;
    const overlay = (
      <div>
        <p>Overlay</p>
        <button onClick={ this.hideOverlay }>hide</button>
      </div>
    );

    return (
      <SectionList
        items={ items }
        overlay={ showOverlay ? overlay : null }
        renderItem={ this.renderItem }
      />
    );
  }

  renderItem (item, index) {
    const { desc, name } = item;

    return (
      <div style={ ITEM_STYLE }>
        <h3>{ name }</h3>
        <h3 data-hover='show'>{ desc }</h3>
      </div>
    );
  }

  hideOverlay = () => {
    this.setState({ showOverlay: false });
  }
}
