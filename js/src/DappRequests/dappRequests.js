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

import { observer } from 'mobx-react';
import React from 'react';

import RequestGroups from './RequestGroups';
import Store from './store';
import styles from './dappRequests.css';

function DappRequests () {
  const store = Store.get();

  console.log(
    'REQUESTS',
    store.requests,
    store.hasRequests,
    store.groupedRequests
  );

  if (!store || !store.hasRequests) {
    return null;
  }

  console.log('REQUESTS', this.groupedRequests);

  return (
    <div className={ styles.requests }>
      {Object.keys(store.groupedRequests)
        .map(appId => (
          <RequestGroups
            key={ appId }
            appId={ appId }
            onApproveRequestGroup={ store.approveRequestGroup }
            onRejectRequestGroup={ store.rejectRequestGroup }
            requestGroups={ store.groupedRequests[appId] }
          />
        ))}
    </div>
  );
}

export default observer(DappRequests);
