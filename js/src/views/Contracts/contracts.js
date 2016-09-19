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

import React, { Component, PropTypes } from 'react';

import Summary from './Summary';

import styles from './contracts.css';

export default class Contracts extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    contracts: PropTypes.array.isRequired
  }

  state = {
  }

  render () {
    return (
      <div>
        <div className={ styles.contracts }>
          { this.renderContracts() }
        </div>
      </div>
    );
  }

  renderContracts () {
    if (!this.context.contracts) {
      return null;
    }

    return this.context.contracts.map((contract, idx) => {
      return (
        <div
          className={ styles.contract }
          key={ contract.address }>
          <Summary
            contract={ contract } />
        </div>
      );
    });
  }
}
