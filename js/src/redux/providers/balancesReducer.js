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

import { handleActions } from 'redux-actions';

const initialState = {
  balances: {},
  tokens: {},
  tokenreg: null,
  tokensFilter: {}
};

export default handleActions({
  setBalances (state, action) {
    const { balances } = action;

    return Object.assign({}, state, { balances });
  },

  setTokens (state, action) {
    const { tokens } = action;

    if (Array.isArray(tokens)) {
      const objTokens = tokens.reduce((_tokens, token) => {
        _tokens[token.address] = token;
        return _tokens;
      }, {});

      return Object.assign({}, state, { tokens: objTokens });
    }

    return Object.assign({}, state, { tokens });
  },

  setTokenImage (state, action) {
    const { tokenAddress, image } = action;
    const { balances } = state;
    const nextBalances = {};

    Object.keys(balances).forEach((address) => {
      const tokenIndex = balances[address].tokens.findIndex((t) => t.token.address === tokenAddress);

      if (tokenIndex === -1 || balances[address].tokens[tokenIndex].value.equals(0)) {
        return;
      }

      const tokens = [].concat(balances[address].tokens);

      tokens[tokenIndex].token = {
        ...tokens[tokenIndex].token,
        image
      };

      nextBalances[address] = {
        ...balances[address],
        tokens
      };
    });

    return Object.assign({}, state, { balance: { ...balances, nextBalances } });
  },

  setTokenReg (state, action) {
    const { tokenreg } = action;

    return Object.assign({}, state, { tokenreg });
  },

  setTokensFilter (state, action) {
    const { tokensFilter } = action;

    return Object.assign({}, state, { tokensFilter });
  }
}, initialState);
