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
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    shorten: PropTypes.bool,
    unknown: PropTypes.bool
  }

  render () {
    const { address, accounts, contacts, contracts, tokens, shorten, unknown } = this.props;
    const account = (accounts || {})[address] || (contacts || {})[address] || (tokens || {})[address] || (contracts || {})[address];
    const addressFallback = shorten ? this.formatHash(address) : address;
    const fallback = unknown ? 'UNNAMED' : addressFallback;
    const isUuid = account && account.name === account.uuid;
    const name = account && !isUuid
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
  const { accounts, contacts, contracts } = state.personal;
  const { tokens } = state.balances;

  return {
    accounts,
    contacts,
    contracts,
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
