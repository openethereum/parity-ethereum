// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import LogLevel from 'loglevel';

import { LOG_KEYS } from '~/config';
import { Select, Container, LanguageSelector } from '~/ui';

import layout from '../layout.css';

export default class Parity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  state = {
    loglevels: {},
    mode: 'active',
    selectValues: []
  };

  componentWillMount () {
    this.loadMode();
    this.loadLogLevels();
    this.setSelectValues();
  }

  loadLogLevels () {
    if (process.env.NODE_ENV === 'production') {
      return null;
    }

    const nextState = { ...this.state.logLevels };

    Object.keys(LOG_KEYS).map((logKey) => {
      const log = LOG_KEYS[logKey];

      const logger = LogLevel.getLogger(log.key);
      const level = logger.getLevel();

      nextState[logKey] = { level, log };
    });

    this.setState({ logLevels: nextState });
  }

  setSelectValues () {
    if (process.env.NODE_ENV === 'production') {
      return null;
    }

    const selectValues = Object.keys(LogLevel.levels).map((levelName) => {
      const value = LogLevel.levels[levelName];

      return {
        name: levelName,
        value
      };
    });

    this.setState({ selectValues });
  }

  render () {
    return (
      <Container
        title={
          <FormattedMessage id='settings.parity.label' />
        }
      >
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>
              <FormattedMessage
                id='settings.parity.overview_0'
                defaultMessage='Control the Parity node settings and mode of operation via this interface.'
              />
            </div>
          </div>
          <div className={ layout.details }>
            <LanguageSelector />
            { this.renderModes() }
          </div>
        </div>

        { this.renderLogsConfig() }
      </Container>
    );
  }

  renderLogsConfig () {
    if (process.env.NODE_ENV === 'production') {
      return null;
    }

    return (
      <div className={ layout.layout }>
        <div className={ layout.overview }>
          <div>
            <FormattedMessage
              id='settings.parity.loglevels'
              defaultMessage='Choose the different logs level.'
            />
          </div>
        </div>
        <div className={ layout.details }>
          { this.renderLogsLevels() }
        </div>
      </div>
    );
  }

  renderLogsLevels () {
    if (process.env.NODE_ENV === 'production') {
      return null;
    }

    const { logLevels, selectValues } = this.state;

    return Object.keys(logLevels).map((logKey) => {
      const { level, log } = logLevels[logKey];
      const { key, desc } = log;

      const onChange = (_, index) => {
        const nextLevel = Object.values(selectValues)[index].value;
        LogLevel.getLogger(key).setLevel(nextLevel);
        this.loadLogLevels();
      };

      return (
        <div key={ logKey }>
          <p>{ desc }</p>
          <Select
            onChange={ onChange }
            value={ level }
            values={ selectValues }
          />
        </div>
      );
    });
  }

  renderModes () {
    const { mode } = this.state;

    const renderItem = (mode, label) => {
      return (
        <MenuItem
          key={ mode }
          value={ mode }
          label={ label }
        >
          { label }
        </MenuItem>
      );
    };

    return (
      <Select
        label={
          <FormattedMessage
            id='settings.parity.modes.label'
            defaultMessage='mode of operation'
          />
        }
        hint={
          <FormattedMessage
            id='settings.parity.modes.hint'
            defaultMessage='the syning mode for the Parity node'
          />
        }
        value={ mode }
        onChange={ this.onChangeMode }
      >
        {
          renderItem('active',
            <FormattedMessage
              id='settings.parity.modes.mode_active'
              defaultMessage='Parity continuously syncs the chain'
            />
          )
        }
        {
          renderItem('passive',
            <FormattedMessage
              id='settings.parity.modes.mode_passive'
              defaultMessage='Parity syncs initially, then sleeps and wakes regularly to resync'
            />
          )
        }
        {
          renderItem('dark',
            <FormattedMessage
              id='settings.parity.modes.mode_dark'
              defaultMessage='Parity syncs only when the RPC is active'
            />
          )
        }
        {
          renderItem('offline',
            <FormattedMessage
              id='settings.parity.modes.mode_offline'
              defaultMessage="Parity doesn't sync"
            />
          )
        }
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
