export const LS_KEY = 'tooltips';

function closeTooltips (state, action) {
  const { currentId } = action;

  window.localStorage.setItem(LS_KEY, '{"state":"off"}');

  return Object.assign({}, state, {
    currentId
  });
}

function newTooltip (state, action) {
  const { currentId, maxId } = action;

  return Object.assign({}, state, {
    currentId,
    maxId
  });
}

function nextTooltip (state, action) {
  const { currentId } = action;

  return Object.assign({}, state, {
    currentId
  });
}

export default function tooltipReducer (state = {}, action) {
  switch (action.type) {
    case 'newTooltip':
      return newTooltip(state, action);

    case 'nextTooltip':
      return nextTooltip(state, action);

    case 'closeTooltips':
      return closeTooltips(state, action);

    default:
      return state;
  }
}
