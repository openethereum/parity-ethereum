// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { bindActionCreators } from 'redux';

class IdentityName extends Component {
  static propTypes = {
    address: PropTypes.string,
    accountsInfo: PropTypes.object,
    tokens: PropTypes.object,
    empty: PropTypes.bool,
    shorten: PropTypes.bool,
    unknown: PropTypes.bool
  }

  render () {
    const { address, accountsInfo, tokens, empty, shorten, unknown } = this.props;
    const account = accountsInfo[address] || tokens[address];
    const hasAccount = account && (!account.meta || !account.meta.deleted);

    if (!hasAccount && empty) {
      return null;
    }

    const addressFallback = shorten ? this.formatHash(address) : address;
    const fallback = unknown ? 'UNNAMED' : addressFallback;
    const isUuid = hasAccount && account.name === account.uuid;
    const name = hasAccount && !isUuid
      ? account.name.toUpperCase()
      : fallback;

    return (
      <span>{ name }</span>
    );
  }

  formatHash (hash) {
    if (!hash || hash.length <= 16) {
      return hash;
    }

    return `${hash.substr(2, 6)}...${hash.slice(-6)}`;
  }
}

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;
  const { tokens } = state.balances;

  return {
    accountsInfo,
    tokens
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(IdentityName);
