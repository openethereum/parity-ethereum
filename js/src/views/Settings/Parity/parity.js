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
import { FormattedMessage } from 'react-intl';
import { MenuItem } from 'material-ui';

import { Select, Container, LanguageSelector } from '~/ui';

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
    return (
      <Container title={
        <FormattedMessage id='settings.parity.label' />
      }>
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>
              <FormattedMessage
                id='settings.parity.overview_0'
                defaultMessage='Control the Parity node settings and mode of operation via this interface.' />
            </div>
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
      const label = <FormattedMessage id={ `settings.parity.modes.mode_${mode}` } />;

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

    return (
      <Select
        label={
          <FormattedMessage
            id='settings.parity.modes.label'
            defaultMessage='mode of operation' />
        }
        hint={
          <FormattedMessage
            id='settings.parity.modes.hint'
            defaultMessage='the syning mode for the Parity node' />
        }
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
