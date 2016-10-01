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

import layout from '../style.css';

export default class Deploy extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  state = {
    deploying: false,
    name: '',
    nameError: null,
    tla: '',
    tlaError: null,
    totalSupply: '1000000',
    totalSupplyError: null
  }

  render () {
    return (
      <div className={ layout.body }>
        <div className={ layout.title }>Deploy</div>
        { this.renderForm() }
      </div>
    );
  }

  renderForm () {
    const { deploying, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;

    if (deploying) {
      return null;
    }

    const error = `${layout.input} ${layout.error}`;

    return (
      <div className={ layout.form }>
        <div className={ nameError ? error : layout.input }>
          <label>token name</label>
          <input
            value={ name }
            onChange={ this.onChangeName } />
          <div className={ layout.hint }>
            A name for the token to identify it
          </div>
        </div>
        <div className={ tlaError ? error : layout.input }>
          <label>token TLA</label>
          <input
            className={ layout.small }
            value={ tla }
            onChange={ this.onChangeTla } />
          <div className={ layout.hint }>
            A unique network acronym for this token (3 characters)
          </div>
        </div>
        <div className={ totalSupplyError ? error : layout.input }>
          <label>total number of tokens</label>
          <input
            type='number'
            min='1000'
            max='999999999'
            value={ totalSupply }
            onChange={ this.onChangeSupply } />
          <div className={ layout.hint }>
            The total number of tokens in circulation
          </div>
        </div>
      </div>
    );
  }

  onChangeName = (event, name) => {
    this.setState({ name });
  }

  onChangeTla = (event, tla) => {
    this.setState({ tla });
  }

  onChangeSupply = (event, totalSupply) => {
    this.setState({ totalSupply });
  }
}
