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

import ReactEventListener from 'react-event-listener';
import React, { Component, PropTypes } from 'react';

let listenerId = 0;
let listenerIds = [];

export default class StackEventListener extends Component {
  static propTypes = {
    onKeyUp: PropTypes.func.isRequired
  };

  componentWillMount () {
    // Add to the list of listeners on mount
    this.id = ++listenerId;
    listenerIds.push(this.id);
  }

  componentWillUnmount () {
    // Remove from the listeners list on unmount
    listenerIds = listenerIds.filter((id) => this.id !== id);
  }

  render () {
    return (
      <ReactEventListener
        target='window'
        onKeyUp={ this.handleKeyUp }
      />
    );
  }

  handleKeyUp = (event) => {
    // Only handle event if last of the listeners list
    if (this.id !== listenerIds.slice(-1)[0]) {
      return event;
    }

    return this.props.onKeyUp(event);
  }
}
