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
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Container } from '~/ui';
import { PauseIcon, PlayIcon, ReorderIcon, ReplayIcon } from '~/ui/Icons';

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
    const { logsLevels } = this.debugStore;

    return (
      <Container
        title={
          <FormattedMessage
            id='status.debug.title'
            defaultMessage='Node Logs'
          />
        }
      >
        { this.renderActions() }
        <h2 className={ styles.subheader }>
          { logsLevels || '-' }
        </h2>
        { this.renderToggle() }
        { this.renderLogs() }
      </Container>
    );
  }

  renderToggle () {
    const { logsEnabled } = this.debugStore;

    if (logsEnabled) {
      return null;
    }

    return (
      <div className={ styles.stopped }>
        <FormattedMessage
          id='status.debug.stopped'
          defaultMessage='Refresh and display of logs from Parity is currently stopped via the UI, start it to see the latest updates.'
        />
      </div>
    );
  }

  renderLogs () {
    const { logs } = this.debugStore;

    if (logs.length === 0) {
      return null;
    }

    const text = logs
      .map((log, index) => {
        return (
          <p key={ index } className={ styles.log }>
            <span className={ styles.logDate }>[{ log.date.toLocaleString() }]</span>
            <span className={ styles.logText }>{ log.log }</span>
          </p>
        );
      });

    return (
      <div className={ styles.logs }>
        { text }
      </div>
    );
  }

  renderActions () {
    const { logsEnabled } = this.debugStore;
    const toggleButton = logsEnabled
      ? <PauseIcon />
      : <PlayIcon />;

    return (
      <div className={ styles.actions }>
        <a onClick={ this.toggle }>{ toggleButton }</a>
        <a onClick={ this.clear }><ReplayIcon /></a>
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
