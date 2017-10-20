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

import accountsReducer from './addresses/accounts-reducer.js';
import contactsReducer from './addresses/contacts-reducer.js';
import lookupReducer from './Lookup/reducers.js';
import eventsReducer from './Events/reducers.js';
import namesReducer from './Names/reducers.js';
import recordsReducer from './Records/reducers.js';
import reverseReducer from './Reverse/reducers.js';

const netVersionReducer = (state = null, action) =>
  action.type === 'set netVersion' ? action.netVersion : state;

const contractReducer = (state = null, action) =>
  action.type === 'set contract' ? action.contract : state;

const feeReducer = (state = null, action) =>
  action.type === 'set fee' ? action.fee : state;

const ownerReducer = (state = null, action) =>
  action.type === 'set owner' ? action.owner : state;

const initialState = {
  netVersion: netVersionReducer(undefined, { type: '' }),
  accounts: accountsReducer(undefined, { type: '' }),
  contacts: contactsReducer(undefined, { type: '' }),
  contract: contractReducer(undefined, { type: '' }),
  fee: feeReducer(undefined, { type: '' }),
  owner: ownerReducer(undefined, { type: '' }),
  lookup: lookupReducer(undefined, { type: '' }),
  events: eventsReducer(undefined, { type: '' }),
  names: namesReducer(undefined, { type: '' }),
  records: recordsReducer(undefined, { type: '' }),
  reverse: reverseReducer(undefined, { type: '' })
};

export default (state = initialState, action) => ({
  netVersion: netVersionReducer(state.netVersion, action),
  accounts: accountsReducer(state.accounts, action),
  contacts: contactsReducer(state.contacts, action),
  contract: contractReducer(state.contract, action),
  fee: feeReducer(state.fee, action),
  owner: ownerReducer(state.owner, action),
  lookup: lookupReducer(state.lookup, action),
  events: eventsReducer(state.events, action),
  names: namesReducer(state.names, action),
  records: recordsReducer(state.records, action),
  reverse: reverseReducer(state.reverse, action)
});
