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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { Container } from '@parity/ui';
import { PauseIcon, PlayIcon, ReorderIcon, ReplayIcon } from '@parity/ui/Icons';

import Logs from './Logs';
import Toggle from './Toggle';
import DebugStore from './store';
import styles from './debug.css';

@observer
export default class Debug extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  debugStore = new DebugStore(this.context.api);

  componentWillUnmount () {
    this.debugStore.stopPolling();
  }

  render () {
    const { logs, logsEnabled, logsLevels } = this.debugStore;

    return (
      <Container
        title={
          <FormattedMessage
            id='status.debug.title'
            defaultMessage='Node Logs'
          />
        }
      >
        <div className={ styles.actions }>
          <a onClick={ this.toggle }>
            {
              logsEnabled
                ? <PauseIcon />
                : <PlayIcon />
            }
          </a>
          <a onClick={ this.clear }>
            <ReplayIcon />
          </a>
          <a
            onClick={ this.reverse }
            title={
              <FormattedMessage
                id='status.debug.reverse'
                defaultMessage='Reverse Order'
              />
            }
          >
            <ReorderIcon />
          </a>
        </div>
        <h2 className={ styles.subheader }>
          { logsLevels || '-' }
        </h2>
        <Toggle logsEnabled={ logsEnabled } />
        <Logs logs={ logs } />
      </Container>
    );
  }

  clear = () => {
    this.debugStore.clearLogs();
  };

  toggle = () => {
    this.debugStore.toggle();
  };

  reverse = () => {
    this.debugStore.reverse();
  };
}
