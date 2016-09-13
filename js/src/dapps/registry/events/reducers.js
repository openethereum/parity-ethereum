const initialState = [];

const sortEvents = (a, b) => {
  const d = a.block.minus(b.block).toFixed(0)
  if (d === 0) return a.index.minus(b.index).toFixed(0)
  return d
}

export default (state = initialState, action) => {
  if (action.type === 'events subscribe start')
    return state; // TODO store the subscriptions?
  if (action.type === 'events subscribe fail')
    return state; // TODO ?

  if (action.type === 'events event') {
    if (action.event.state !== 'mined') return state
    return state
      .filter((event) => event.key !== action.event.key)
      .concat(action.event)
      .sort(sortEvents);
  }

  return state;
};
