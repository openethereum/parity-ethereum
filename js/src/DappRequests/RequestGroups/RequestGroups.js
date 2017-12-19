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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import DappsStore from '@parity/shared/lib/mobx/dappsStore';

import RequestGroupSubItem from './RequestGroupSubItem';
import styles from './RequestGroups.css';

export default class RequestGroups extends Component {
  handleApproveRequestGroup = (requests, groupId) => {
    this.props.onApproveRequestGroup(requests, groupId, this.props.appId);
  }

  render () {
    const {
      appId,
      requestGroups,
      onRejectRequestGroup
    } = this.props;

    const app = DappsStore.get().getAppById(appId);

    return (
      <div className={ styles.requestGroups }>
        <FormattedMessage
          className={ styles.requestGroups }
          id='dappRequests.request.info'
          defaultMessage='{appName} wants to request permissions:'
          values={ {
            appName: app ? app.name : `Dapp ${appId}`
          } }
        />
        {Object.keys(requestGroups).map(groupId => (
          <RequestGroupSubItem
            key={ `${appId}-${groupId}` }
            groupId={ groupId }
            requests={ requestGroups[groupId] }
            onApprove={ this.handleApproveRequestGroup }
            onReject={ onRejectRequestGroup }
          />
        ))}
      </div>
    );
  }
}

RequestGroups.propTypes = {
  appId: PropTypes.string.isRequired,
  className: PropTypes.string,
  onApproveRequestGroup: PropTypes.func.isRequired,
  onRejectRequestGroup: PropTypes.func.isRequired,
  requestGroups: PropTypes.object.isRequired
};
