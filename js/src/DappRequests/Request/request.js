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

import Button from '@parity/ui/lib/Button';

import DappsStore from '@parity/shared/lib/mobx/dappsStore';

export default function Request ({ appId, className, approveRequest, denyRequest, queueId, request: { from, method } }) {
  const _onApprove = () => approveRequest(queueId, false);
  const _onApproveAll = () => approveRequest(queueId, true);
  const _onReject = () => denyRequest(queueId);

  const app = DappsStore.get().getAppById(appId);

  return (
    <div className={ className }>
      <FormattedMessage
        id='dappRequests.request.info'
        defaultMessage='Received request for {method} from {appName}'
        values={ {
          appName:
            app
              ? app.name
              : appId,
          method
        } }
      />
      <div>
        <Button
          label={
            <FormattedMessage
              id='dappRequests.request.buttons.approve'
              defaultMessage='Approve'
            />
          }
          onClick={ _onApprove }
        />
        <Button
          label={
            <FormattedMessage
              id='dappRequests.request.buttons.approveAll'
              defaultMessage='Approve All'
            />
          }
          onClick={ _onApproveAll }
        />
        <Button
          label={
            <FormattedMessage
              id='dappRequests.request.buttons.reject'
              defaultMessage='Reject'
            />
          }
          onClick={ _onReject }
        />
      </div>
    </div>
  );
}

Request.propTypes = {
  appId: PropTypes.string.isRequired,
  className: PropTypes.string,
  approveRequest: PropTypes.func.isRequired,
  denyRequest: PropTypes.func.isRequired,
  queueId: PropTypes.number.isRequired,
  request: PropTypes.object.isRequired
};
