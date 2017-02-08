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
    origin: PropTypes.oneOfType([
      PropTypes.oneOf(['unknown']),
      PropTypes.shape({ dapp: PropTypes.string.isRequired }),
      PropTypes.shape({ rpc: PropTypes.string.isRequired }),
      PropTypes.shape({ ipc: PropTypes.string.isRequired }),
      PropTypes.shape({ signer: PropTypes.string.isRequired })
    ]).isRequired
  };

  render () {
    const { origin } = this.props;

    return (
      <div className={ styles.container }>
        Request Origin: { this.renderOrigin(origin) }
      </div>
    );
  }

  renderOrigin (origin) {
    if (origin === 'unknown') {
      return (
        <span className={ styles.unknown }>unknown</span>
      );
    }

    if ('dapp' in origin) {
      return (
        <span>
          Dapp at <span className={ styles.url }>
            { origin.dapp || 'unknown URL' }
          </span>
        </span>
      );
    }

    if ('rpc' in origin) {
      return (
        <span>
          RPC <span className={ styles.url }>
            ({ origin.rpc || 'unidentified' })
          </span>
        </span>
      );
    }

    if ('ipc' in origin) {
      return (
        <span>
          IPC session
          <span
            className={ styles.hash }
            title={ origin.ipc }
          >
            <IdentityIcon
              address={ origin.ipc }
              tiny
            />
          </span>
        </span>
      );
    }

    if ('signer' in origin) {
      return this.renderSigner(origin.signer);
    }
  }

  renderSigner (session) {
    if (session.substr(2) === this.context.api.transport.sessionHash) {
      return (
        <span title={ session }>Current Tab</span>
      );
    }

    return (
      <span>
        UI session
        <span
          className={ styles.hash }
          title={ session }
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
