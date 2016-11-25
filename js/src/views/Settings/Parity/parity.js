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

import { Select, Container, ContainerTitle, LanguageSelector, Translate } from '../../../ui';

import layout from '../layout.css';

const MODES = ['active', 'passive', 'dark', 'offline'];

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
    const title = <Translate id='settings.parity.label' />;

    return (
      <Container>
        <ContainerTitle title={ title } />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div><Translate id='settings.parity.overview_0' /></div>
          </div>
          <div className={ layout.details }>
            <LanguageSelector />
            { this.renderModes() }
          </div>
        </div>
      </Container>
    );
  }

  renderModes () {
    const modes = MODES.map((mode) => {
      const label = <Translate id={ `settings.parity.modes.mode_${mode}` } />;

      return (
        <MenuItem
          key={ mode }
          value={ mode }
          label={ label }>
          { label }
        </MenuItem>
      );
    });
    const { mode } = this.state;
    const label = <Translate id='settings.parity.modes.label' />;
    const hint = <Translate id='settings.parity.modes.hint' />;

    return (
      <Select
        label={ label }
        hint={ hint }
        value={ mode }
        onChange={ this.onChangeMode }>
        { modes }
      </Select>
    );
  }

  onChangeMode = (event, index, mode) => {
    const { api } = this.context;

    api.parity
      .setMode(mode)
      .then((result) => {
        if (result) {
          this.setState({ mode });
        }
      })
      .catch((error) => {
        console.warn('onChangeMode', error);
      });
  }

  loadMode () {
    const { api } = this.context;

    api.parity
      .mode()
      .then((mode) => {
        this.setState({ mode });
      })
      .catch((error) => {
        console.warn('loadMode', error);
      });
  }
}
