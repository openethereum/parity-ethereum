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

const ERRORS = {
  requireSender: 'A valid sender is required for the transaction',
  requireRecipient: 'A recipient network address is required for the transaction',
  invalidAddress: 'The supplied address is an invalid network address',
  invalidAmount: 'The supplied amount should be a valid positive number',
  invalidDecimals: 'The supplied amount exceeds the allowed decimals',
  largeAmount: 'The transaction total is higher than the available balance',
  gasException: 'The transaction will throw an exception with the current values',
  gasBlockLimit: 'The transaction execution will exceed the block gas limit'
};

export default ERRORS;
