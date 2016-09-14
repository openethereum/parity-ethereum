const initialState = {
  subscriptions: {
    Reserved: null,
    Dropped: null
  },
  pending: {
    Reserved: false,
    Dropped: false
  },
  events: []
};

const sortEvents = (a, b) => {
  const d = a.block.minus(b.block).toFixed(0);
  if (d === 0) return a.index.minus(b.index).toFixed(0);
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
    console.warn('events unsubscribe', action);
    return {
      ...state,
      pending: { ...state.pending, [action.name]: false },
      subscriptions: { ...state.subscriptions, [action.name]: null },
      events: state.events.filter((event) => event.type !== action.name)
    };
  }

  if (action.type === 'events event' && action.event.state === 'mined') {
    return { ...state, events: state.events
      .filter((event) => event.key !== action.event.key)
      .concat(action.event)
      .sort(sortEvents)
    };
  }

  return state;
};
