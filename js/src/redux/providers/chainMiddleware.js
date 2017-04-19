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
          const { nodeStatus } = store.getState();
          const { netChain, nodeKind } = nodeStatus;
          const newChain = collection.netChain;
          const newNodeKind = collection.nodeKind;
          let reloadChain = false;
          let reloadType = false;

          // force reload when chain has changed and is not initial value
          if (newChain) {
            const hasChainChanged = newChain !== netChain;
            const isInitialChain = netChain === DEFAULT_NETCHAIN;

            reloadChain = !isInitialChain && hasChainChanged;
          }

          // force reload when nodeKind (availability or capability) has changed
          if (newNodeKind && nodeKind) {
            const hasAvailabilityChanged = nodeKind.availability !== newNodeKind.availability;
            const hasCapabilityChanged = nodeKind.capability !== newNodeKind.capability;

            reloadType = hasAvailabilityChanged || hasCapabilityChanged;
          }

          if (reloadChain || reloadType) {
            setTimeout(() => {
              window.location.reload();
            }, 0);
          }
        }
      }

      next(action);
    };
  }
}
