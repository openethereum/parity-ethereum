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
import React, { PureComponent } from 'react';

import RequestGroups from './RequestGroups';
import Store from './store';
import styles from './dappRequests.css';

class DappRequests extends PureComponent {
  store = Store.get();

  handleApproveRequestGroup = requestIds => {
    requestIds.forEach(this.store.approveRequest);
  }

  handleRejectRequestGroup = requestIds => {
    requestIds.forEach(this.store.rejectRequest);
  }

  render () {
    if (!this.store || !this.store.hasRequests) {
      return null;
    }

    return (
      <div className={ styles.requests }>
        {Object.keys(this.store.groupedRequests)
          .map(appId => (
            <RequestGroups
              key={ appId }
              appId={ appId }
              onApproveRequestGroup={ this.handleApproveRequestGroup }
              onRejectRequestGroup={ this.handleRejectRequestGroup }
              requestGroups={ this.store.groupedRequests[appId] }
            />
          ))}
      </div>
    );
  }
}

export default observer(DappRequests);
