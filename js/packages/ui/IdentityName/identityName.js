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
import { connect } from 'react-redux';

import { isNullAddress } from '@parity/shared/util/validation';

import ShortenedHash from '../ShortenedHash';

const defaultName = (
  <FormattedMessage
    id='ui.identityName.unnamed'
    defaultMessage='UNNAMED'
  />
);
const defaultNameNull = (
  <FormattedMessage
    id='ui.identityName.null'
    defaultMessage='NULL'
  />
);

export function IdentityName ({ account, address, className, empty, name, shorten, unknown }) {
  if (!account && empty) {
    return null;
  }

  const nullName = isNullAddress(address) ? defaultNameNull : null;
  const addressFallback = nullName || (shorten ? (<ShortenedHash data={ address } />) : address);
  const fallback = unknown ? defaultName : addressFallback;
  const isUuid = account && account.name === account.uuid;
  const displayName = (name && name.toUpperCase().trim()) ||
    (account && !isUuid
      ? account.name.toUpperCase().trim()
      : fallback
    );

  return (
    <span className={ className }>
      {
        displayName && displayName.length
          ? displayName
          : fallback
      }
    </span>
  );
}

IdentityName.propTypes = {
  account: PropTypes.object,
  address: PropTypes.string,
  className: PropTypes.string,
  empty: PropTypes.bool,
  name: PropTypes.string,
  shorten: PropTypes.bool,
  unknown: PropTypes.bool
};

function mapStateToProps (state, props) {
  const { address } = props;
  const { personal, tokens } = state;

  const account = personal.accountsInfo[address] || Object.values(tokens).find((token) => token.address === address);

  return {
    account
  };
}

export default connect(
  mapStateToProps,
  null
)(IdentityName);
