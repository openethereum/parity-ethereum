// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

// no need for react since not using JSX
import React, { Component, PropTypes } from 'react';

export default Wrapped => class Web3Compositor extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  tickActive = false

  render () {
    return (
      <Wrapped { ...this.props } ref={ this.registerComponent } />
    );
  }

  componentDidMount () {
    this.tickActive = true;
    setTimeout(this.next);
  }

  componentWillUnmount () {
    this.tickActive = false;
  }

  next = () => {
    if (!this.tickActive) {
      return;
    }

    if (!this.wrapped || !this.wrapped.onTick) {
      setTimeout(this.next, 5000);
      return;
    }

    let nextCalled = false;
    this.wrapped.onTick(error => {
      if (nextCalled) {
        return;
      }
      nextCalled = true;
      setTimeout(this.next, error ? 10000 : 2000);
    });
  }

  registerComponent = component => {
    this.wrapped = component;
  }

};
