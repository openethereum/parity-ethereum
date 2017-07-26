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
import { observer } from 'mobx-react';

import Store from '../NetChain/store';

const SYMBOL_ETC = 'ETC';
const SYMBOL_ETH = 'ETH';
const SYMBOL_EXP = 'EXP';

function renderSymbol (netChain) {
  switch (netChain) {
    case 'classic':
      return SYMBOL_ETC;

    case 'expanse':
      return SYMBOL_EXP;

    default:
      return SYMBOL_ETH;
  }
}

function CurrencySymbol ({ className }, { api }) {
  const store = Store.get(api);

  return (
    <span className={ className }>
      { renderSymbol(store.netChain) }
    </span>
  );
}

CurrencySymbol.propTypes = {
  className: PropTypes.string
};

CurrencySymbol.contextTypes = {
  api: PropTypes.object.isRequired
};

export default observer(CurrencySymbol);
