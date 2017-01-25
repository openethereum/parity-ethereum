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

const SYMBOL_ETC = 'ETC';
const SYMBOL_ETH = 'ETH';
const SYMBOL_EXP = 'EXP';

class CurrencySymbol extends Component {
  static propTypes = {
    className: PropTypes.string,
    netChain: PropTypes.string.isRequired,
    netSymbol: PropTypes.string.isRequired
  }

  render () {
    const { className, netSymbol } = this.props;

    return (
      <span className={ className }>{ netSymbol }</span>
    );
  }
}

function mapStateToProps (state) {
  const { netChain } = state.nodeStatus;
  let netSymbol;

  switch (netChain) {
    case 'classic':
      netSymbol = SYMBOL_ETC;
      break;

    case 'expanse':
      netSymbol = SYMBOL_EXP;
      break;

    default:
      netSymbol = SYMBOL_ETH;
      break;
  }

  return {
    netChain,
    netSymbol
  };
}

export default connect(
  mapStateToProps,
  null
)(CurrencySymbol);
