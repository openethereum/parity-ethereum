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

import React, { PureComponent } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import Popup from 'semantic-ui-react/dist/commonjs/modules/Popup';
import Button from '@parity/ui/lib/Button';

import methodGroups from '../../methodGroups';
import styles from './RequestGroupSubItem.css';

export default class RequestGroupSubItem extends PureComponent {
  handleApprove = () => this.props.onApprove(this.props.requests, this.props.groupId)

  handleReject = () => this.props.onReject(this.props.requests)

  render () {
    const { groupId } = this.props;

    return (
      <div className={ styles.requestGroupSubItem }>
        <span className={ styles.requestGroupSubItemTitle }>
          Permission for{' '}
          <Popup
            trigger={ <span>{groupId}</span> }
            content={ `Requested methods: ${methodGroups[groupId].methods.join(', ')}` }
          />
        </span>
        <Button
          size='mini'
          label={
            <FormattedMessage
              id='dappRequests.request.buttons.approve'
              defaultMessage='Approve'
            />
          }
          onClick={ this.handleApprove }
        />
        <Button
          size='mini'
          label={
            <FormattedMessage
              id='dappRequests.request.buttons.reject'
              defaultMessage='Reject'
            />
          }
          onClick={ this.handleReject }
        />
      </div>

    );
  }
}

RequestGroupSubItem.propTypes = {
  className: PropTypes.string,
  groupId: PropTypes.string,
  onApprove: PropTypes.func.isRequired,
  onReject: PropTypes.func.isRequired,
  requests: PropTypes.array.isRequired
};
