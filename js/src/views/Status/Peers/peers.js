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
import { connect } from 'react-redux';

import { Container, ContainerTitle } from '~/ui';

import styles from './peers.css';

class Peers extends Component {
  static propTypes = {
    peers: PropTypes.array.isRequired
  };

  render () {
    const { peers } = this.props;

    return (
      <Container>
        <ContainerTitle
          title={
            <FormattedMessage
              id='status.peers.title'
              defaultMessage='network peers'
            />
          }
        />
        <table className={ styles.peers }>
          <tbody>
            { this.renderPeers(peers) }
          </tbody>
        </table>
      </Container>
    );
  }

  renderPeers (peers) {
    return peers.map((peer) => this.renderPeer(peer));
  }

  renderPeer (peer) {
    const { caps, id, name, network, protocols } = peer;

    return (
      <tr
        className={ styles.peer }
        key={ id }
      >
        <td>
          { id }
        </td>
        <td>{ name }</td>
      </tr>
    );
  }
}

function mapStateToProps (state) {
  const { netPeers } = state.nodeStatus;
  const { peers = [] } = netPeers;
  const realPeers = peers
    .filter((peer) => peer.id)
    .sort((peerA, peerB) => peerA.id.localeCompare(peerB.id));

  return { peers: realPeers };
}

export default connect(mapStateToProps)(Peers);
