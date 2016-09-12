const initialState = [];

export default (state = initialState, action) => {
  if (action.type === 'events subscribe start')
    return state; // TODO store the subscriptions?
  if (action.type === 'events subscribe fail')
    return state; // TODO ?

  if (action.type === 'events event')
    return state
      .filter((event) => event.key !== action.key)
      .concat(action.event);

  return state;
};
