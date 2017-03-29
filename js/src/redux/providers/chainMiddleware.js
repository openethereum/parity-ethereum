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

import { DEFAULT_NETCHAIN } from './statusReducer';

export default class ChainMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      if (action.type === 'statusCollection') {
        const { collection } = action;

        if (collection) {
          const newChain = collection.netChain;
          const newNodeKind = collection.nodeKind;

          if (newChain) {
            const { nodeStatus } = store.getState();
            const { netChain, nodeKind } = nodeStatus;

            // force reload when chain has changed
            let forceReload = newChain !== netChain && netChain !== DEFAULT_NETCHAIN;

            // force reload when nodeKind (availability/capability) has changed
            if (!forceReload && collection.nodeKind !== null && nodeKind !== null) {
              forceReload = nodeKind.availability !== newNodeKind.availability ||
                nodeKind.capability !== newNodeKind.capability;
            }

            if (forceReload) {
              setTimeout(() => {
                window.location.reload();
              }, 0);
            }
          }
        }
      }

      next(action);
    };
  }
}
