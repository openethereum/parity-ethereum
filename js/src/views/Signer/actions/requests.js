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

import { withError } from '../../../redux/util';
import { identity } from '../components/util/util';

import { createAction } from 'redux-actions';

// TODO [legacy;todr] Remove
export const updateCompatibilityMode = createAction('update compatibilityMode');

export const updatePendingRequests = createAction('update pendingRequests');
export const startConfirmRequest = createAction('start confirmRequest');
export const successConfirmRequest = createAction('success confirmRequest');
export const errorConfirmRequest = createAction('error confirmRequest', identity,
  withError(args => args.err, 'error')
);
export const startRejectRequest = createAction('start rejectRequest');
export const successRejectRequest = createAction('success rejectRequest');
export const errorRejectRequest = createAction('error rejectRequest', identity,
  withError(args => args.err, 'error')
);
