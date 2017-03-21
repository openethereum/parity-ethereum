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

import Call from '../Call';
import CallsToolbar from '../CallsToolbar';
import styles from './Calls.css';

export default class Calls extends Component {
  state = {
    activeCall: null,
    activeChild: null
  }

  render () {
    return (
      <div
        className='calls-container'
        onMouseLeave={ this.clearActiveCall }
        { ...this._test('container') }
      >
        { this.renderClear() }
        <h2 className={ styles.header }>
          <FormattedMessage
            id='status.calls.title'
            defaultMessage='History'
          />
        </h2>
        <div className={ `${styles.history} row` } ref={ this.setCallsHistory }>
          { this.renderNoCallsMsg() }
          { this.renderCalls() }
        </div>
        <CallsToolbar
          call={ this.state.activeCall }
          callEl={ this.state.activeChild }
          containerEl={ this._callsHistory }
          actions={ this.props.actions }
        />
      </div>
    );
  }

  renderClear () {
    if (!this.props.calls.length) {
      return;
    }

    return (
      <a
        { ...this._test('remove') }
        title={
          <FormattedMessage
            id='status.calls.clearHistory'
            defaultMessage='Clear RPC calls history'
          />
        }
        onClick={ this.clearHistory }
        className={ styles.removeIcon }
      >
        <i className='icon-trash' />
      </a>
    );
  }

  renderNoCallsMsg () {
    if (this.props.calls.length) {
      return;
    }

    return (
      <div { ...this._test('empty-wrapper') }>
        <h3 className={ styles.historyInfo } { ...this._test('empty') }>
          <FormattedMessage
            id='status.calls.rpcResults'
            defaultMessage='Fire up some calls and the results will be here.'
          />
        </h3>
      </div>
    );
  }

  renderCalls () {
    const { calls } = this.props;

    if (!calls.length) {
      return;
    }

    return calls.map((call, idx) => (
      <Call
        key={ calls.length - idx }
        call={ call }
        setActiveCall={ this.setActiveCall }
      />
    ));
  }

  clearActiveCall = () => {
    this.setState({ activeCall: null, activeChild: null });
  }

  setActiveCall = (call, el) => {
    this.setState({ activeCall: call, activeChild: el });
  }

  setCallsHistory = el => {
    this._callsHistory = el;
  }

  clearHistory = () => {
    this.props.reset();
  }

  static propTypes = {
    calls: PropTypes.arrayOf(PropTypes.object).isRequired,
    actions: PropTypes.shape({
      fireRpc: PropTypes.func.isRequired,
      copyToClipboard: PropTypes.func.isRequired,
      selectRpcMethod: PropTypes.func.isRequired
    }).isRequired,
    reset: PropTypes.func
  }
}
