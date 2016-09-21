import accountsReducer from './addresses/accounts-reducer.js';
import contactsReducer from './addresses/contacts-reducer.js';
import lookupReducer from './Lookup/reducers.js';
import eventsReducer from './events/reducers.js';
import registerReducer from './register/reducers.js';
import recordsReducer from './records/reducer.js';

const contractReducer = (state = null, action) =>
  action.type === 'set contract' ? action.contract : state;

const feeReducer = (state = null, action) =>
  action.type === 'set fee' ? action.fee : state;

const ownerReducer = (state = null, action) =>
  action.type === 'set owner' ? action.owner : state;

const initialState = {
  accounts: accountsReducer(undefined, { type: '' }),
  contacts: contactsReducer(undefined, { type: '' }),
  contract: contractReducer(undefined, { type: '' }),
  fee: feeReducer(undefined, { type: '' }),
  owner: ownerReducer(undefined, { type: '' }),
  lookup: lookupReducer(undefined, { type: '' }),
  events: eventsReducer(undefined, { type: '' }),
  register: registerReducer(undefined, { type: '' }),
  records: recordsReducer(undefined, { type: '' })
};

export default (state = initialState, action) => ({
  accounts: accountsReducer(state.accounts, action),
  contacts: contactsReducer(state.contacts, action),
  contract: contractReducer(state.contract, action),
  fee: feeReducer(state.fee, action),
  owner: ownerReducer(state.owner, action),
  lookup: lookupReducer(state.lookup, action),
  events: eventsReducer(state.events, action),
  register: registerReducer(state.register, action),
  records: recordsReducer(state.records, action)
});
