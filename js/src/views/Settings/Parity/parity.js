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
    this.store.loadChain();
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
                defaultMessage='Control the Parity node settings and nature of syncing via this interface.'
              />
            </div>
          </div>
          <div className={ layout.details }>
            { this.renderChains() }
            { this.renderModes() }
            <Features />
            <LanguageSelector />
          </div>
        </div>
        { this.renderLogsConfig() }
      </Container>
    );
  }

  renderItem (name, label) {
    return (
      <MenuItem
        key={ name }
        label={ label }
        value={ name }
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
            defaultMessage='the syncing mode for the Parity node'
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

  renderChains () {
    const { chain } = this.store;

    return (
      <Select
        id='parityChainSelect'
        hint={
          <FormattedMessage
            id='settings.parity.chains.hint'
            defaultMessage='the chain for the Parity node to sync to'
          />
        }
        label={
          <FormattedMessage
            id='settings.parity.chains.label'
            defaultMessage='chain/network to sync'
          />
        }
        onChange={ this.onChangeChain }
        value={ chain }
      >
        {
          this.renderItem('foundation', (
            <FormattedMessage
              id='settings.parity.chains.chain_foundation'
              defaultMessage='Parity syncs to the Ethereum network launched by the Ethereum Foundation'
            />
          ))
        }
        {
          this.renderItem('kovan', (
            <FormattedMessage
              id='settings.parity.chains.chain_kovan'
              defaultMessage='Parity syncs to the Kovan test network'
            />
          ))
        }
        {
          this.renderItem('olympic', (
            <FormattedMessage
              id='settings.parity.chains.chain_olympic'
              defaultMessage='Parity syncs to the Olympic test network'
            />
          ))
        }
        {
          this.renderItem('morden', (
            <FormattedMessage
              id='settings.parity.chains.cmorden_kovan'
              defaultMessage='Parity syncs to Morden (Classic) test network'
            />
          ))
        }
        {
          this.renderItem('ropsten', (
            <FormattedMessage
              id='settings.parity.chains.chain_ropsten'
              defaultMessage='Parity syncs to the Ropsten test network'
            />
          ))
        }
        {
          this.renderItem('classic', (
            <FormattedMessage
              id='settings.parity.chains.chain_classic'
              defaultMessage='Parity syncs to the Ethereum Classic network'
            />
          ))
        }
        {
          this.renderItem('expanse', (
            <FormattedMessage
              id='settings.parity.chains.chain_expanse'
              defaultMessage='Parity syncs to the Expanse network'
            />
          ))
        }
        {
          this.renderItem('dev', (
            <FormattedMessage
              id='settings.parity.chains.chain_dev'
              defaultMessage='Parity uses a local development chain'
            />
          ))
        }
      </Select>
    );
  }

  onChangeMode = (event, index, mode) => {
    this.store.changeMode(mode || event.target.value);
  }

  onChangeChain = (event, index, chain) => {
    this.store.changeChain(chain || event.target.value);
  }
}
