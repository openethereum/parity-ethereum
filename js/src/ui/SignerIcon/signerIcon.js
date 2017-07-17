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
import { connect } from 'react-redux';
import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

import IdentityIcon from '../IdentityIcon';
import { FingerprintIcon } from '../Icons';

function SignerIcon ({ className, secureToken }) {
  if (!secureToken) {
    return (
      <FingerprintIcon />
    );
  }

  const signerSha = keccak_256(secureToken);

  return (
    <IdentityIcon
      address={ signerSha }
      center
      className={ className }
    />
  );
}

SignerIcon.propTypes = {
  className: PropTypes.string,
  secureToken: PropTypes.string
};

function mapStateToProps (state) {
  const { secureToken } = state.nodeStatus;

  return { secureToken };
}

export default connect(
  mapStateToProps,
  null
)(SignerIcon);
