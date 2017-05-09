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

import { createAction } from 'redux-actions';

import { withError } from '../util';

function identity (x) {
  return x;
}

export function signerRequestsToConfirm (pending) {
  return {
    type: 'signerRequestsToConfirm',
    pending
  };
}

export const startConfirmRequest = createAction('signerStartConfirmRequest');
export const successConfirmRequest = createAction('signerSuccessConfirmRequest');
export const errorConfirmRequest = createAction('signerErrorConfirmRequest', identity,
  withError((args) => args.err, 'error')
);
export const startRejectRequest = createAction('signerStartRejectRequest');
export const successRejectRequest = createAction('signerSuccessRejectRequest');
export const errorRejectRequest = createAction('signerErrorRejectRequest', identity,
  withError((args) => args.err, 'error')
);
