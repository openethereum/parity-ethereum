const initialState = {
  contract: null,
  fee: null,
  owner: null
};

export default (state = initialState, action) => {
  if (action.type === 'set contract')
    return { ...state, contract: action.contract };

  if (action.type === 'set fee')
    return { ...state, fee: action.fee };

  if (action.type === 'set owner')
    return { ...state, owner: action.owner };

  return state;
};
