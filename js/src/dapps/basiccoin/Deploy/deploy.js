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

import Container from '../Container';
import styles from './deploy.css';
import layout from '../style.css';

const ERRORS = {
  name: 'specify a valid name >3 & <32 characters',
  tla: 'specify a valid TLA, 3 characters in length'
};

export default class Deploy extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  state = {
    deploying: false,
    name: '',
    nameError: ERRORS.name,
    tla: '',
    tlaError: ERRORS.tla,
    totalSupply: '1000000',
    totalSupplyError: null
  }

  render () {
    const { deploying } = this.state;

    return deploying
      ? this.renderDeploying()
      : this.renderForm();
  }

  renderDeploying () {
    return (
      <Container center>
        Your token is currently being deployed to the network
      </Container>
    );
  }

  renderForm () {
    const { name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);
    const error = `${layout.input} ${layout.error}`;

    return (
      <Container>
        <div className={ layout.form }>
          <div className={ nameError ? error : layout.input }>
            <label>token name</label>
            <input
              value={ name }
              name='name'
              onChange={ this.onChangeName } />
            <div className={ layout.hint }>
              { nameError || 'an identifying name for the token' }
            </div>
          </div>
          <div className={ tlaError ? error : layout.input }>
            <label>token TLA</label>
            <input
              className={ layout.small }
              name='tla'
              value={ tla }
              onChange={ this.onChangeTla } />
            <div className={ layout.hint }>
              { tlaError || 'unique network acronym for this token' }
            </div>
          </div>
          <div className={ totalSupplyError ? error : layout.input }>
            <label>total number of tokens</label>
            <input
              type='number'
              min='1000'
              max='999999999'
              name='totalSupply'
              value={ totalSupply }
              onChange={ this.onChangeSupply } />
            <div className={ layout.hint }>
              { totalSupplyError || 'The total number of tokens in circulation' }
            </div>
          </div>
        </div>
        <div className={ styles.buttonRow }>
          <div
            className={ styles.button }
            disabled={ hasError }
            onClick={ this.onDeploy }>
            Deploy Token
          </div>
        </div>
      </Container>
    );
  }

  onChangeName = (event) => {
    const name = event.target.value;
    const nameError = name && (name.length > 3) && (name.length < 32)
      ? null
      : ERRORS.name;

    this.setState({ name, nameError });
  }

  onChangeTla = (event) => {
    const _tla = event.target.value;
    const tla = _tla && (_tla.length > 3)
      ? _tla.substr(0, 3)
      : _tla;
    const tlaError = tla && (tla.length === 3)
      ? null
      : ERRORS.tla;

    this.setState({ tla, tlaError });
  }

  onChangeSupply = (event) => {
    const totalSupply = event.target.value;

    this.setState({ totalSupply });
  }

  onDeploy = (event) => {
    const { deploying, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);

    if (hasError || deploying) {
      return;
    }

    this.setState({ deploying: true });
  }
}
