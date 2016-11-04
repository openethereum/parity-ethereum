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
import { MenuItem } from 'material-ui';

import { Select, Container, ContainerTitle } from '../../../ui';

import layout from '../layout.css';

const MODES = {
  'active': 'active',
  'passive': 'passive',
  'dark': 'dark',
  'off': 'off'
};

export default class Parity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    mode: 'active'
  }

  componentWillMount () {
    this.loadMode();
  }

  render () {
    return (
      <Container>
        <ContainerTitle title='Parity' />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>Control the Parity node settings via this interface.</div>
          </div>
          <div className={ layout.details }>
            { this.renderModes() }
          </div>
        </div>
      </Container>
    );
  }

  renderModes () {
    const modes = Object
      .keys(MODES)
      .map((mode) => {
        const description = MODES[mode];

        return (
          <MenuItem
            key={ mode }
            value={ mode }
            label={ description }>
            { description }
          </MenuItem>
        );
      });

    const { mode } = this.state;

    return (
      <Select
        label='mode of operation'
        hint='the syning mode for the Parity node'
        value={ mode }
        onChange={ this.onChangeMode }>
        { modes }
      </Select>
    );
  }

  onChangeMode = (event, mode) => {
    this.setState({ mode });
  }

  loadMode () {
    const { api } = this.context;

    api.ethcore
      .mode()
      .then((mode) => {
        this.setMode({ mode });
      })
      .catch((error) => {
        console.warn('loadMode', error);
      });
  }
}
