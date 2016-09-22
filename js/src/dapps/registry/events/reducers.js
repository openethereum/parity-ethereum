const initialState = {
  subscriptions: {
    Reserved: null,
    Dropped: null,
    DataChanged: null
  },
  pending: {
    Reserved: false,
    Dropped: false,
    DataChanged: false
  },
  events: []
};

const sortEvents = (a, b) => {
  if (a.state === 'pending' && b.state !== 'pending') return -1;
  if (a.state !== 'pending' && b.state === 'pending') return 1;
  const d = b.block.minus(a.block).toFixed(0);
  if (d === 0) return b.index.minus(a.index).toFixed(0);
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
