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

import { Dropdown, Menu } from 'semantic-ui-react';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Select, Container, LanguageSelector } from '~/ui';
import Features, { FeaturesStore, FEATURES } from '~/ui/Features';

import Store, { LOGLEVEL_OPTIONS } from './store';
import layout from '../layout.css';

@observer
export default class Node extends Component {
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

  renderItem (name, func, label) {
    return (
      <Dropdown.Item
        key={ name }
        content={ label }
        name={ name }
        onClick={ func }
      >
        { label }
      </Dropdown.Item>
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
      <div className={ layout.option }>
        <FormattedMessage
          id='settings.parity.modes.hint'
          defaultMessage='Choose the syncing mode for the Parity node'
        />
        <Menu
          vertical
          id='parityModeSelect'
          value={ mode }
        >
          <Dropdown
            item
            text={ mode }
          >
            <Dropdown.Menu>
              {
                this.renderItem('active', this.onChangeMode, (
                  <FormattedMessage
                    id='settings.parity.modes.mode_active'
                    defaultMessage='Continuously sync'
                  />
                ))
              }
              {
                this.renderItem('passive', this.onChangeMode, (
                  <FormattedMessage
                    id='settings.parity.modes.mode_passive'
                    defaultMessage='Sync on intervals'
                  />
                ))
              }
              {
                this.renderItem('dark', this.onChangeMode, (
                  <FormattedMessage
                    id='settings.parity.modes.mode_dark'
                    defaultMessage='Sync when RPC is active'
                  />
                ))
              }
              {
                this.renderItem('offline', this.onChangeMode, (
                  <FormattedMessage
                    id='settings.parity.modes.mode_offline'
                    defaultMessage='No Syncing'
                  />
                ))
              }
            </Dropdown.Menu>
          </Dropdown>
        </Menu>
      </div>
    );
  }

  renderChains () {
    const { chain } = this.store;

    return (
      <div className={ layout.option }>
        <FormattedMessage
          id='settings.parity.chains.hint'
          defaultMessage='Choose the chain for the Parity node to sync to'
        />
        <Menu
          vertical
          id='parityChainSelect'
          value={ chain }
        >
          <Dropdown item text={ chain }>
            <Dropdown.Menu>
              {
                this.renderItem('foundation', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_foundation'
                    defaultMessage='Ethereum Foundation'
                  />
                ))
              }
              {
                this.renderItem('kovan', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_kovan'
                    defaultMessage='Kovan test network'
                  />
                ))
              }
              {
                this.renderItem('olympic', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_olympic'
                    defaultMessage='Olympic test network'
                  />
                ))
              }
              {
                this.renderItem('morden', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.cmorden_kovan'
                    defaultMessage='Morden (Classic) test network'
                  />
                ))
              }
              {
                this.renderItem('ropsten', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_ropsten'
                    defaultMessage='Ropsten test network'
                  />
                ))
              }
              {
                this.renderItem('classic', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_classic'
                    defaultMessage='Ethereum Classic network'
                  />
                ))
              }
              {
                this.renderItem('expanse', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_expanse'
                    defaultMessage='Expanse network'
                  />
                ))
              }
              {
                this.renderItem('dev', this.onChangeChain, (
                  <FormattedMessage
                    id='settings.parity.chains.chain_dev'
                    defaultMessage='Local development chain'
                  />
                ))
              }
            </Dropdown.Menu>
          </Dropdown>
        </Menu>
      </div>
    );
  }

  onChangeMode = (e, mode) => {
    this.store.changeMode(mode.name);
  }

  onChangeChain = (e, chain) => {
    this.store.changeChain(chain.name);
  }
}
