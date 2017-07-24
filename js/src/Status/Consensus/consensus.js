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

import React from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

export default function Consensus ({ upgradeStore }) {
  if (!upgradeStore || !upgradeStore.consensusCapability) {
    return null;
  }

  if (upgradeStore.consensusCapability === 'capable') {
    return (
      <div>
        <FormattedMessage
          id='application.status.consensus.capable'
          defaultMessage='Capable'
        />
      </div>
    );
  }

  if (upgradeStore.consensusCapability.capableUntil) {
    return (
      <div>
        <FormattedMessage
          id='application.status.consensus.capableUntil'
          defaultMessage='Capable until #{blockNumber}'
          values={ {
            blockNumber: upgradeStore.consensusCapability.capableUntil
          } }
        />
      </div>
    );
  }

  if (upgradeStore.consensusCapability.incapableSince) {
    return (
      <div>
        <FormattedMessage
          id='application.status.consensus.incapableSince'
          defaultMessage='Incapable since #{blockNumber}'
          values={ {
            blockNumber: upgradeStore.consensusCapability.incapableSince
          } }
        />
      </div>
    );
  }

  return (
    <div>
      <FormattedMessage
        id='application.status.consensus.unknown'
        defaultMessage='Unknown capability'
      />
    </div>
  );
}

Consensus.propTypes = {
  upgradeStore: PropTypes.object.isRequired
};
