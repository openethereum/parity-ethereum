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
import CopyToClipboard from 'react-copy-to-clipboard';
import { FormattedMessage } from 'react-intl';
import { sortBy, find, extend } from 'lodash';

import IconButton from 'material-ui/IconButton';
import MoreHorizIcon from 'material-ui/svg-icons/navigation/more-horiz';
import CallIcon from 'material-ui/svg-icons/communication/call';
import AssignmentIcon from 'material-ui/svg-icons/action/assignment';
import InputIcon from 'material-ui/svg-icons/action/input';

import { SCROLLBAR_WIDTH } from '../../constants';
import styles from './CallsToolbar.css';
import rpcData from '../../data/rpc.json';
const rpcMethods = sortBy(rpcData.methods, 'name');

export default class CallsToolbar extends Component {
  render () {
    const { call, callEl, containerEl } = this.props;

    if (!call) {
      return null;
    }

    const wrapStyle = { top: callEl.offsetTop - SCROLLBAR_WIDTH - containerEl.scrollTop };

    if (this.hasScrollbar(containerEl)) {
      wrapStyle.right = 13;
    }

    return (
      <div
        className={ styles.callActionsWrap }
        style={ wrapStyle }
      >
        <IconButton
          className={ styles.callActionsButton }
          { ...this._test('button-more') }
        >
          <MoreHorizIcon />
        </IconButton>
        <div className={ styles.callActions } { ...this._test('button-container') }>
          <IconButton
            className={ styles.callAction }
            onTouchTap={ this.setCall }
            tooltip={
              <FormattedMessage
                id='status.callsToolbar.tooltip.set'
                defaultMessage='Set'
              />
            }
            tooltipPosition='top-left'
            { ...this._test('button-setCall') }
          >
            <InputIcon className={ styles.callActionIcon } />
          </IconButton>
          <IconButton
            className={ styles.callAction }
            onTouchTap={ this.makeCall }
            tooltip={
              <FormattedMessage
                id='status.callsToolbar.tooltip.fireAgain'
                defaultMessage='Fire again'
              />
            }
            tooltipPosition='top-left'
            { ...this._test('button-makeCall') }
          >
            <CallIcon className={ styles.callActionIcon } />
          </IconButton>
          <CopyToClipboard
            text={ JSON.stringify(call) }
            onCopy={ this.copyToClipboard }
          >
            <IconButton
              className={ styles.callAction }
              tooltip={
                <FormattedMessage
                  id='status.callsToolbar.tooltip.copy'
                  defaultMessage='Copy to clipboard'
                />
              }
              tooltipPosition='top-left'
              { ...this._test('copyCallToClipboard') }
            >
              <AssignmentIcon className={ styles.callActionIcon } />
            </IconButton>
          </CopyToClipboard>
        </div>
      </div>
    );
  }

  setCall = () => {
    const { call } = this.props;
    let method = find(rpcMethods, { name: call.name });

    this.props.actions.selectRpcMethod(extend({}, method, { paramsValues: call.params }));
  }

  makeCall = () => {
    const { call } = this.props;
    let method = find(rpcMethods, { name: call.name });

    this.setCall();
    this.props.actions.fireRpc({
      method: method.name,
      outputFormatter: method.outputFormatter,
      inputFormatters: method.inputFormatters,
      params: call.params
    });
  }

  copyToClipboard = () => {
    this.props.actions.copyToClipboard(
      <FormattedMessage
        id='status.callsToolbar.copied'
        defaultMessage='method copied to clipboard'
      />
    );
  }

  hasScrollbar (el) {
    return el.clientHeight < el.scrollHeight;
  }
}

CallsToolbar.propTypes = {
  call: PropTypes.object.isRequired,
  callEl: PropTypes.node.isRequired,
  containerEl: PropTypes.node.isRequired,
  actions: PropTypes.shape({
    fireRpc: PropTypes.func.isRequired,
    copyToClipboard: PropTypes.func.isRequired,
    selectRpcMethod: PropTypes.func.isRequired
  }).isRequired
};
