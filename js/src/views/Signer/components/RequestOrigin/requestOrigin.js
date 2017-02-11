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

import IdentityIcon from '~/ui/IdentityIcon';

import styles from './requestOrigin.css';

export default class RequestOrigin extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    origin: PropTypes.shape({
      type: PropTypes.oneOf(['unknown', 'dapp', 'rpc', 'ipc', 'signer']),
      details: PropTypes.string.isRequired
    }).isRequired
  };

  render () {
    const { origin } = this.props;

    return (
      <div className={ styles.container }>
        Requested { this.renderOrigin(origin) }
      </div>
    );
  }

  renderOrigin (origin) {
    if (origin.type === 'unknown') {
      return (
        <span className={ styles.unknown }>via unknown interface</span>
      );
    }

    if (origin.type === 'dapp') {
      return (
        <span>
          by a dapp at <span className={ styles.url }>
            { origin.details || 'unknown URL' }
          </span>
        </span>
      );
    }

    if (origin.type === 'rpc') {
      return (
        <span>
          via RPC <span className={ styles.url }>
            ({ origin.details || 'unidentified' })
          </span>
        </span>
      );
    }

    if (origin.type === 'ipc') {
      return (
        <span>
          via IPC session
          <span
            className={ styles.hash }
            title={ origin.details }
          >
            <IdentityIcon
              address={ origin.details }
              tiny
            />
          </span>
        </span>
      );
    }

    if (origin.type === 'signer') {
      return this.renderSigner(origin.details);
    }
  }

  renderSigner (session) {
    if (session.substr(2) === this.context.api.transport.sessionHash) {
      return (
        <span title={ session }>via current tab</span>
      );
    }

    return (
      <span>
        via UI session
        <span
          className={ styles.hash }
          title={ `UI Session id: ${session}` }
        >
          <IdentityIcon
            address={ session }
            tiny
          />
        </span>
      </span>
    );
  }
}
