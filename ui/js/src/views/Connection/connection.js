// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionCompareArrows from 'material-ui/svg-icons/action/compare-arrows';
import ActionDashboard from 'material-ui/svg-icons/action/dashboard';
// import CommunicationVpnKey from 'material-ui/svg-icons/communication/vpn-key';
import HardwareDesktopMac from 'material-ui/svg-icons/hardware/desktop-mac';
import NotificationVpnLock from 'material-ui/svg-icons/notification/vpn-lock';

import { Input } from '../../ui';

import styles from './connection.css';

class Connection extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    isConnected: PropTypes.bool,
    isConnecting: PropTypes.bool,
    isPingable: PropTypes.bool,
    needsToken: PropTypes.bool
  }

  state = {
    token: '',
    validToken: false
  }

  render () {
    const { isConnected, isConnecting, isPingable } = this.props;
    const isOk = !isConnecting && isConnected && isPingable;

    if (isOk) {
      return null;
    }

    const typeIcon = isPingable
      ? <NotificationVpnLock className={ styles.svg } />
      : <ActionDashboard className={ styles.svg } />;
    const description = isPingable
      ? this.renderSigner()
      : this.renderPing();

    return (
      <div>
        <div className={ styles.overlay } />
        <div className={ styles.modal }>
          <div className={ styles.body }>
            <div className={ styles.icons }>
              <div className={ styles.icon }>
                <HardwareDesktopMac className={ styles.svg } />
              </div>
              <div className={ styles.iconSmall }>
                <ActionCompareArrows className={ styles.svg + ' ' + styles.pulse } />
              </div>
              <div className={ styles.icon }>
                { typeIcon }
              </div>
            </div>
            { description }
          </div>
        </div>
      </div>
    );
  }

  renderSigner () {
    const { api } = this.context;
    const { token, validToken } = this.state;
    const { needsToken, isConnecting } = api;

    if (needsToken && !isConnecting) {
      return (
        <div className={ styles.info }>
          <div>Unable to make a connection to the Parity Secure API. To update your secure token or to generate a new one, run <span className={ styles.console }>parity signer new-token</span> and supply the token below</div>
          <div className={ styles.form }>
            <Input
              label='secure token'
              hint='a generated token from Parity'
              error={ validToken || (!token || !token.length) ? null : 'invalid signer token' }
              value={ token }
              onChange={ this.onChangeToken } />
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.info }>
        Connecting to the Parity Secure API.
      </div>
    );
  }

  renderPing () {
    return (
      <div className={ styles.info }>
        Connecting to the Parity Node. If this informational message persists, please ensure that your Parity node is running and reachable on the network.
      </div>
    );
  }

  onChangeToken = (event, token) => {
    const validToken = /[a-zA-Z0-9]{4}-[a-zA-Z0-9]{4}-[a-zA-Z0-9]{4}-[a-zA-Z0-9]{4}/.test(token);
    this.setState({ token, validToken }, () => {
      validToken && this.setToken();
    });
  }

  setToken = () => {
    const { api } = this.context;
    const { token } = this.state;

    api.updateToken(token);
    this.setState({ token: '', validToken: false });
  }
}

function mapStateToProps (state) {
  const { isConnected, isConnecting, isPingable, needsToken } = state.nodeStatus;

  return { isConnected, isConnecting, isPingable, needsToken };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Connection);
