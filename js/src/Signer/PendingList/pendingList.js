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

import BigNumber from 'bignumber.js';
import React from 'react';
import { FormattedMessage } from 'react-intl';
import PropTypes from 'prop-types';

import PendingItem from '../PendingItem';

import styles from './pendingList.css';

export default function PendingList ({ accounts, className, gasLimit, netVersion, onConfirm, onReject, pendingItems }) {
  if (!pendingItems.length) {
    return (
      <div className={ `${styles.none} ${className}` }>
        <FormattedMessage
          id='signer.embedded.noPending'
          defaultMessage='There are currently no pending requests awaiting your confirmation'
        />
      </div>
    );
  }

  return (
    <div className={ `${styles.list} ${className}` }>
      {
        pendingItems
          .sort((a, b) => new BigNumber(a.id).cmp(b.id))
          .map((data, index) => (
            <PendingItem
              accounts={ accounts }
              data={ data }
              gasLimit={ gasLimit }
              isFocussed={ index === 0 }
              key={ data.id }
              netVersion={ netVersion }
              onConfirm={ onConfirm }
              onReject={ onReject }
            />
          ))
      }
    </div>
  );
}

PendingList.propTypes = {
  accounts: PropTypes.object.isRequired,
  className: PropTypes.string,
  gasLimit: PropTypes.object.isRequired,
  netVersion: PropTypes.string.isRequired,
  onConfirm: PropTypes.func.isRequired,
  onReject: PropTypes.func.isRequired,
  pendingItems: PropTypes.array.isRequired
};
