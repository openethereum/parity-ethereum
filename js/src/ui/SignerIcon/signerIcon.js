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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';

import IdentityIcon from '../IdentityIcon';

class SignerIcon extends Component {
  static propTypes = {
    className: PropTypes.string,
    secureToken: PropTypes.string
  }

  render () {
    const { className, secureToken } = this.props;

    if (!secureToken) {
      return (
        <ActionFingerprint />
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
}

function mapStateToProps (state) {
  const { secureToken } = state.nodeStatus;

  return { secureToken };
}

export default connect(
  mapStateToProps,
  null
)(SignerIcon);
