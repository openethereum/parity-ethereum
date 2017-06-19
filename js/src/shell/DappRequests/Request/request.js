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

import { Button } from '@parity/ui';

export default function Request ({ className, approveRequest, denyRequest, queueId, request: { from, method } }) {
  const _onApprove = () => approveRequest(queueId, false);
  const _onApproveAll = () => approveRequest(queueId, true);
  const _onReject = () => denyRequest(queueId);

  return (
    <div className={ className }>
      <FormattedMessage
        id='dappRequests.request.info'
        defaultMessage='Received request for {method} from {from}'
        values={ {
          from,
          method
        } }
      />
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
  );
}

Request.propTypes = {
  className: PropTypes.string,
  approveRequest: PropTypes.func.isRequired,
  denyRequest: PropTypes.func.isRequired,
  queueId: PropTypes.number.isRequired,
  request: PropTypes.object.isRequired
};
