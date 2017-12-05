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

const initialState = {
  subscriptions: {
    Reserved: null,
    Dropped: null,
    DataChanged: null,
    ReverseProposed: null,
    ReverseConfirmed: null,
    ReverseRemoved: null
  },
  pending: {
    Reserved: false,
    Dropped: false,
    DataChanged: false,
    ReverseProposed: false,
    ReverseConfirmed: false,
    ReverseRemoved: false
  },
  events: []
};

const sortEvents = (a, b) => {
  if (a.state === 'pending' && b.state !== 'pending') {
    return -1;
  } else if (a.state !== 'pending' && b.state === 'pending') {
    return 1;
  }

  const d = b.block.minus(a.block).toFixed(0);

  if (d === 0) {
    return b.index.minus(a.index).toFixed(0);
  }

  return d;
};

export default (state = initialState, action) => {
  if (!(action.name in state.subscriptions)) { // invalid event name
    return state;
  }

  if (action.type === 'events subscribe start') {
    return { ...state, pending: { ...state.pending, [action.name]: true } };
  }
  if (action.type === 'events subscribe fail') {
    return { ...state, pending: { ...state.pending, [action.name]: false } };
  }
  if (action.type === 'events subscribe success') {
    return {
      ...state,
      pending: { ...state.pending, [action.name]: false },
      subscriptions: { ...state.subscriptions, [action.name]: action.subscription }
    };
  }

  if (action.type === 'events unsubscribe') {
    return {
      ...state,
      pending: { ...state.pending, [action.name]: false },
      subscriptions: { ...state.subscriptions, [action.name]: null },
      events: state.events.filter((event) => event.type !== action.name)
    };
  }

  if (action.type === 'events event') {
    return { ...state, events: state.events
      .filter((event) => event.key !== action.event.key)
      .concat(action.event)
      .sort(sortEvents)
    };
  }

  return state;
};
