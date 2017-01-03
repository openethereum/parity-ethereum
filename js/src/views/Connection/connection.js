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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionCompareArrows from 'material-ui/svg-icons/action/compare-arrows';
import ActionDashboard from 'material-ui/svg-icons/action/dashboard';
import HardwareDesktopMac from 'material-ui/svg-icons/hardware/desktop-mac';
import NotificationVpnLock from 'material-ui/svg-icons/notification/vpn-lock';

import { Input } from '~/ui';

import styles from './connection.css';

class Connection extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    isConnected: PropTypes.bool,
    isConnecting: PropTypes.bool,
    needsToken: PropTypes.bool
  }

  state = {
    loading: false,
    token: '',
    validToken: false
  }

  render () {
    const { isConnected, needsToken } = this.props;

    if (isConnected) {
      return null;
    }

    const typeIcon = needsToken
      ? <NotificationVpnLock className={ styles.svg } />
      : <ActionDashboard className={ styles.svg } />;
    const description = needsToken
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
                <ActionCompareArrows className={ `${styles.svg} ${styles.pulse}` } />
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
    const { loading, token, validToken } = this.state;
    const { isConnecting, needsToken } = this.props;

    if (needsToken && !isConnecting) {
      return (
        <div className={ styles.info }>
          <div>
            <FormattedMessage
              id='connection.noConnection'
              defaultMessage='Unable to make a connection to the Parity Secure API. To update your secure token or to generate a new one, run {newToken} and supply the token below'
              values={ {
                newToken: <span className={ styles.console }>parity signer new-token</span>
              } } />
          </div>
          <div className={ styles.form }>
            <Input
              disabled={ loading }
              error={
                validToken || (!token || !token.length)
                  ? null
                  : (
                    <FormattedMessage
                      id='connection.invalidToken'
                      defaultMessage='invalid signer token' />
                  )
              }
              hint={
                <FormattedMessage
                  id='connection.token.hint'
                  defaultMessage='a generated token from Parity' />
              }
              label={
                <FormattedMessage
                  id='connection.token.label'
                  defaultMessage='secure token' />
              }
              onChange={ this.onChangeToken }
              value={ token } />
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.info }>
        <FormattedMessage
          id='connection.connectingAPI'
          defaultMessage='Connecting to the Parity Secure API.' />
      </div>
    );
  }

  renderPing () {
    return (
      <div className={ styles.info }>
        <FormattedMessage
          id='connection.connectingNode'
          defaultMessage='Connecting to the Parity Node. If this informational message persists, please ensure that your Parity node is running and reachable on the network.' />
      </div>
    );
  }

  onChangeToken = (event, _token) => {
    const token = _token.trim();
    const validToken = /^[a-zA-Z0-9]{4}(-)?[a-zA-Z0-9]{4}(-)?[a-zA-Z0-9]{4}(-)?[a-zA-Z0-9]{4}$/.test(token);

    this.setState({ token, validToken }, () => {
      validToken && this.setToken();
    });
  }

  setToken = () => {
    const { api } = this.context;
    const { token } = this.state;

    this.setState({ loading: true });

    api
      .updateToken(token, 0)
      .then((isValid) => {
        this.setState({
          loading: isValid || false,
          validToken: isValid
        });
      });
  }
}

function mapStateToProps (state) {
  const { isConnected, isConnecting, needsToken } = state.nodeStatus;

  return { isConnected, isConnecting, needsToken };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Connection);
