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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Container } from '~/ui';
import { PauseIcon, PlayIcon, ReorderIcon, ReplayIcon } from '~/ui/Icons';

import styles from './debug.css';

export default class Debug extends Component {
  static propTypes = {
    actions: PropTypes.shape({
      clearStatusLogs: PropTypes.func.isRequired,
      toggleStatusLogs: PropTypes.func.isRequired
    }).isRequired,
    nodeStatus: PropTypes.object.isRequired
  }

  state = {
    reversed: true
  }

  render () {
    const { nodeStatus } = this.props;
    const { devLogsLevels } = nodeStatus;

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
          { devLogsLevels || '-' }
        </h2>
        { this.renderToggle() }
        { this.renderLogs() }
      </Container>
    );
  }

  renderToggle () {
    const { devLogsEnabled } = this.props.nodeStatus;

    if (devLogsEnabled) {
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
    const { nodeStatus } = this.props;
    const { reversed } = this.state;
    const { devLogs } = nodeStatus;

    const dateRegex = /^(\d{4}.\d{2}.\d{2}.\d{2}.\d{2}.\d{2})(.*)$/i;

    if (!devLogs) {
      return null;
    }

    const logs = reversed
      ? [].concat(devLogs).reverse()
      : [].concat(devLogs);

    const text = logs
      .map((log, index) => {
        const logDate = dateRegex.exec(log);

        if (!logDate) {
          return (
            <p key={ index } className={ styles.log }>
              { log }
            </p>
          );
        }

        return (
          <p key={ index } className={ styles.log }>
            <span className={ styles.logDate }>{ logDate[1] }</span>
            <span className={ styles.logText }>{ logDate[2] }</span>
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
    const { devLogsEnabled } = this.props.nodeStatus;
    const toggleButton = devLogsEnabled
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
    const { clearStatusLogs } = this.props.actions;

    clearStatusLogs();
  }

  toggle = () => {
    const { devLogsEnabled } = this.props.nodeStatus;
    const { toggleStatusLogs } = this.props.actions;

    toggleStatusLogs(!devLogsEnabled);
  }

  reverse = () => {
    const { reversed } = this.state;

    this.setState({ reversed: !reversed });
  }
}
