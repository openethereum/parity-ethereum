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

import React, { Component } from 'react';
import LogLevel from 'loglevel';

import { Container, Select } from '~/ui';
import { LOG_KEYS } from '~/config';

import layout from '../layout.css';

export default class AdvancedSettings extends Component {

  state = {
    loglevels: {},
    selectValues: []
  };

  componentWillMount () {
    this.loadLogLevels();

    const selectValues = Object.keys(LogLevel.levels).map((levelName) => {
      const value = LogLevel.levels[levelName];

      return {
        name: levelName,
        value
      };
    });

    this.setState({ selectValues });
  }

  loadLogLevels () {
    const nextState = { ...this.state.logLevels };

    Object.keys(LOG_KEYS).map((logKey) => {
      const log = LOG_KEYS[logKey];

      const logger = LogLevel.getLogger(log.path);
      const level = logger.getLevel();

      nextState[logKey] = { level, log };
    });

    this.setState({ logLevels: nextState });
  }

  render () {
    return (
      <Container title='Advanced Settings'>
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>
              Choose the different logs level.
            </div>
          </div>
          <div className={ layout.details }>
            { this.renderLogsLevels() }
          </div>
        </div>
      </Container>
    );
  }

  renderLogsLevels () {
    const { logLevels, selectValues } = this.state;

    return Object.keys(logLevels).map((logKey) => {
      const { level, log } = logLevels[logKey];
      const { path, desc } = log;

      const onChange = (_, index) => {
        const nextLevel = Object.values(selectValues)[index].value;
        LogLevel.getLogger(path).setLevel(nextLevel);
        this.loadLogLevels();
      };

      return (
        <div key={ logKey }>
          <div>{ path }</div>
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
}
