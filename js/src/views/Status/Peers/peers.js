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

import React, { PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { Container } from '@parity/ui';

import Peer from './Peer';

import styles from './peers.css';

function Peers ({ peers }) {
  return (
    <Container
      title={
        <FormattedMessage
          id='status.peers.title'
          defaultMessage='network peers'
        />
      }
    >
      <div className={ styles.peers }>
        <table>
          <thead>
            <tr>
              <th />
              <th>
                <FormattedMessage
                  id='status.peers.table.header.id'
                  defaultMessage='ID'
                />
              </th>
              <th>
                <FormattedMessage
                  id='status.peers.table.header.remoteAddress'
                  defaultMessage='Remote Address'
                />
              </th>
              <th>
                <FormattedMessage
                  id='status.peers.table.header.name'
                  defaultMessage='Name'
                />
              </th>
              <th>
                <FormattedMessage
                  id='status.peers.table.header.ethHeader'
                  defaultMessage='Header (ETH)'
                />
              </th>
              <th>
                <FormattedMessage
                  id='status.peers.table.header.ethDiff'
                  defaultMessage='Difficulty (ETH)'
                />
              </th>
              <th>
                <FormattedMessage
                  id='status.peers.table.header.caps'
                  defaultMessage='Capabilities'
                />
              </th>
            </tr>
          </thead>
          <tbody>
            {
              peers.map((peer, index) => {
                return (
                  <Peer
                    index={ index }
                    key={ index }
                    peer={ peer }
                  />
                );
              })
            }
          </tbody>
        </table>
      </div>
    </Container>
  );
}

Peers.propTypes = {
  peers: PropTypes.array.isRequired
};

function mapStateToProps (state) {
  const handshakeRegex = /handshake/i;

  const { netPeers } = state.nodeStatus;
  const { peers = [] } = netPeers;
  const realPeers = peers
    .filter((peer) => peer.id)
    .filter((peer) => !handshakeRegex.test(peer.network.remoteAddress))
    .filter((peer) => peer.protocols.eth && peer.protocols.eth.head)
    .sort((peerA, peerB) => {
      const idComp = peerA.id.localeCompare(peerB.id);

      return idComp;
    });

  return { peers: realPeers };
}

export default connect(
  mapStateToProps,
  null
)(Peers);
