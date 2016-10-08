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

import logger from '../utils/logger';
import { updateIsConnected } from '../actions/signer';

export default class SignerDataProvider {
  constructor (store, ws) {
    this.store = store;
    this.ws = ws;

    this.ws.onOpen.push(this.onWsOpen);
    this.ws.onError.push(this.onWsError);
    this.ws.onClose.push(this.onWsError);
  }

  onWsOpen = () => {
    logger.log('[Signer Provider] connected');
    this.store.dispatch(updateIsConnected(true));
  }

  onWsError = () => {
    logger.log('[Signer Provider] error');
    this.store.dispatch(updateIsConnected(false));
  }
}
