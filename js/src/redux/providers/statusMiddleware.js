// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { clearStatusLogs } from './statusActions';
import { loadTokens } from './balancesActions';
import { showSnackbar } from './snackbarActions';
import { clearCertifiers } from './certifications/actions';

export default class SignerMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      if (action.type !== 'statusCollection') {
        next(action);
        return;
      }

      const { collection } = action;
      if (collection && collection.netChain) {
        const state = store.getState();
        if (collection.netChain !== state.nodeStatus.netChain) {
          const chain = collection.netChain;

          state.api
            .parity
            .allAccountsInfo()
            .then((accounts) => {
              showSnackbar(`Switched to ${chain} chain.`, 3000);
              store.dispatch(clearStatusLogs());
              store.dispatch(loadTokens());
              store.dispatch(clearCertifiers());
            })
            .catch((err) => {
              console.error('Failed to refresh accounts:', err);
              store.dispatch(showSnackbar('Failed to refresh accounts.', 3000));
            });
        }
      }

      next(action);
    };
  }
}
