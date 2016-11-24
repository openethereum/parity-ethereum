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

import { handleActions } from 'redux-actions';

const initialState = {
  balances: {},
  tokens: {},

  tokenreg: null
};

export default handleActions({
  setBalances (state, action) {
    const nextBalances = action.balances;
    const prevBalances = state.balances;
    const balances = { ...prevBalances };

    Object.keys(nextBalances).forEach((address) => {
      if (!balances[address]) {
        balances[address] = Object.assign({}, nextBalances[address]);
        return;
      }

      const balance = Object.assign({}, balances[address]);

      const { tokens, txCount = balance.txCount } = nextBalances[address];

      const nextTokens = [].concat(balance.tokens);

      tokens.forEach((t) => {
        const { token, value } = t;
        const { name, tag, image, id, format } = token;

        const tokenIndex = nextTokens.findIndex((tok) => tok.token.tag === tag);

        if (tokenIndex === -1) {
          nextTokens.push({
            token: { name, tag, image, id, format },
            value
          });
        } else {
          nextTokens[tokenIndex] = { token, value };
        }
      });

      balances[address] = Object.assign({}, { txCount, tokens: nextTokens });
    });

    return Object.assign({}, state, { balances });
  },

  setTokens (state, action) {
    const { tokens } = action;
    return Object.assign({}, state, { tokens });
  },

  setTokenReg (state, action) {
    const { tokenreg } = action;
    return Object.assign({}, state, { tokenreg });
  }
}, initialState);
