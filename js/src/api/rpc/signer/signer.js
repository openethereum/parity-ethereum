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

import { inData, inNumber16, inOptions } from '../../format/input';
import { outSignerRequest } from '../../format/output';

export default class Signer {
  constructor (transport) {
    this._transport = transport;
  }

  confirmRequest (requestId, options, password) {
    return this._transport
      .execute('signer_confirmRequest', inNumber16(requestId), inOptions(options), password);
  }

  confirmRequestRaw (requestId, data) {
    return this._transport
      .execute('signer_confirmRequestRaw', inNumber16(requestId), inData(data));
  }

  confirmRequestWithToken (requestId, options, password) {
    return this._transport
      .execute('signer_confirmRequestWithToken', inNumber16(requestId), inOptions(options), password);
  }

  generateAuthorizationToken () {
    return this._transport
      .execute('signer_generateAuthorizationToken');
  }

  generateWebProxyAccessToken (domain) {
    return this._transport
      .execute('signer_generateWebProxyAccessToken', domain);
  }

  rejectRequest (requestId) {
    return this._transport
      .execute('signer_rejectRequest', inNumber16(requestId));
  }

  requestsToConfirm () {
    return this._transport
      .execute('signer_requestsToConfirm')
      .then((requests) => (requests || []).map(outSignerRequest));
  }

  signerEnabled () {
    return this._transport
      .execute('signer_signerEnabled');
  }
}
