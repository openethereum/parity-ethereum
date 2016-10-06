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
import ActionDone from 'material-ui/svg-icons/action/done';
import ContentClear from 'material-ui/svg-icons/content/clear';

import * as styles from './styles';

class Connection extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    secureToken: PropTypes.string,
    isApiConnected: PropTypes.bool,
    isPingConnected: PropTypes.bool
  }

  state = {
    generatingToken: false,
    manualToken: false
  }

  render () {
    const { isApiConnected, isPingConnected, secureToken } = this.props;
    const isSecured = secureToken && secureToken !== 'initial';
    const isConnected = isApiConnected && isPingConnected && isSecured;

    if (isConnected) {
      return null;
    }

    return (
      <div>
        <div style={ styles.overlay } />
        <div style={ styles.modal }>
          <div style={ styles.body }>
            <div style={ styles.icons }>
              { this.renderIcon(isPingConnected, 'Node') }
              { this.renderIcon(isApiConnected, 'API') }
            </div>
            { isPingConnected ? this.renderSigner() : this.renderPing() }
          </div>
        </div>
      </div>
    );
  }

  renderSigner () {
    const { isApiConnected, secureToken } = this.props;
    let details = null;

    return (
      <div style={ styles.info }>
        Connecting to the Parity Secure API. { details }
      </div>
    );
  }

  renderPing () {
    return (
      <div style={ styles.info }>
        Connecting to the Parity Node. If this informational message persists, please ensure that your Parity node is running and reachable on the network.
      </div>
    );
  }

  renderIcon (connected, name) {
    const icon = connected
      ? <ActionDone style={ styles.iconSvg } />
      : <ContentClear style={ styles.iconSvg } />;

    return (
      <div style={ styles.icon }>
        { icon }
        <div style={ styles.iconName }>
          { name }
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { secureToken, isApiConnected, isPingConnected } = state.nodeStatus;

  return {
    secureToken,
    isApiConnected,
    isPingConnected
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Connection);
