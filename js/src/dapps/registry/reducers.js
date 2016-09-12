import lookupReducer from './Lookup/reducers.js';
import eventsReducer from './events/reducers.js';

const initialState = {
  contract: null,
  fee: null,
  owner: null,
  lookup: lookupReducer(undefined, { type: '' }),
  events: eventsReducer(undefined, { type: '' })
};

export default (state = initialState, action) => {
  if (action.type === 'set contract')
    return { ...state, contract: action.contract };

  if (action.type === 'set fee')
    return { ...state, fee: action.fee };

  if (action.type === 'set owner')
    return { ...state, owner: action.owner };

  if (action.type.slice(0, 6) === 'lookup') {
    return { ...state, lookup: lookupReducer(state.lookup, action) };
  }

  if (action.type.slice(0, 6) === 'events') {
    return { ...state, events: eventsReducer(state.events, action) };
  }

  return state;
};
