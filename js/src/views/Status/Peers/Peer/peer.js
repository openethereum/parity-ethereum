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

import { ScrollableText, ShortenedHash } from '@parity/ui';

import styles from '../peers.css';

export default function Peer ({ index, peer }) {
  const { caps, id, name, network, protocols } = peer;

  return (
    <tr
      className={ styles.peer }
      key={ id }
    >
      <td>
        { index + 1 }
      </td>
      <td>
        <ScrollableText small text={ id } />
      </td>
      <td>
        { network.remoteAddress }
      </td>
      <td>
        { name }
      </td>
      <td>
        {
          protocols.eth
            ? <ShortenedHash data={ protocols.eth.head } />
            : null
        }
      </td>
      <td>
        {
          protocols.eth && protocols.eth.difficulty.gt(0)
            ? protocols.eth.difficulty.toExponential(16)
            : null
        }
      </td>
      <td>
        {
          caps && caps.length > 0
            ? caps.join(' - ')
            : null
        }
      </td>
    </tr>
  );
}

Peer.propTypes = {
  peer: PropTypes.object,
  index: PropTypes.number
};
