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

import { MenuItem } from 'material-ui';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Select, Container, LanguageSelector } from '~/ui';
import Features, { FeaturesStore, FEATURES } from '~/ui/Features';

import Store, { LOGLEVEL_OPTIONS } from './store';
import layout from '../layout.css';

@observer
export default class Parity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  store = new Store(this.context.api);
  features = FeaturesStore.get();

  componentWillMount () {
    return this.store.loadMode();
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
            { this.renderModes() }
            <Features />
            <LanguageSelector />
          </div>
        </div>
        { this.renderLogsConfig() }
      </Container>
    );
  }

  renderItem (mode, label) {
    return (
      <MenuItem
        key={ mode }
        label={ label }
        value={ mode }
      >
        { label }
      </MenuItem>
    );
  }

  renderLogsConfig () {
    if (!this.features.active[FEATURES.LOGLEVELS]) {
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
    const { logLevels } = this.store;

    return Object
      .keys(logLevels)
      .map((key) => {
        const { level, log } = logLevels[key];
        const { desc } = log;

        const onChange = (_, index) => {
          this.store.updateLoggerLevel(log.key, Object.values(LOGLEVEL_OPTIONS)[index].value);
        };

        return (
          <div key={ key }>
            <p>{ desc }</p>
            <Select
              onChange={ onChange }
              value={ level }
              values={ LOGLEVEL_OPTIONS }
            />
          </div>
        );
      });
  }

  renderModes () {
    const { mode } = this.store;

    return (
      <Select
        id='parityModeSelect'
        hint={
          <FormattedMessage
            id='settings.parity.modes.hint'
            defaultMessage='the syning mode for the Parity node'
          />
        }
        label={
          <FormattedMessage
            id='settings.parity.modes.label'
            defaultMessage='mode of operation'
          />
        }
        onChange={ this.onChangeMode }
        value={ mode }
      >
        {
          this.renderItem('active', (
            <FormattedMessage
              id='settings.parity.modes.mode_active'
              defaultMessage='Parity continuously syncs the chain'
            />
          ))
        }
        {
          this.renderItem('passive', (
            <FormattedMessage
              id='settings.parity.modes.mode_passive'
              defaultMessage='Parity syncs initially, then sleeps and wakes regularly to resync'
            />
          ))
        }
        {
          this.renderItem('dark', (
            <FormattedMessage
              id='settings.parity.modes.mode_dark'
              defaultMessage='Parity syncs only when the RPC is active'
            />
          ))
        }
        {
          this.renderItem('offline', (
            <FormattedMessage
              id='settings.parity.modes.mode_offline'
              defaultMessage="Parity doesn't sync"
            />
          ))
        }
      </Select>
    );
  }

  onChangeMode = (event, index, mode) => {
    this.store.changeMode(mode || event.target.value);
  }
}
