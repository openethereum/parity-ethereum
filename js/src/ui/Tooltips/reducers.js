const LS_KEY = 'tooltips';

let currentId = -1;
let maxId = 0;

function closeTooltips (state, action) {
  window.localStorage.setItem(LS_KEY, '{"state":"off"}');

  currentId = -1;

  return Object.assign({}, state, {
    currentId
  });
}

function newTooltip (state, action) {
  const { newId } = action;

  maxId = Math.max(newId, maxId);

  return Object.assign({}, state, {
    currentId,
    maxId
  });
}

function nextTooltip (state, action) {
  const hideTips = window.localStorage.getItem(LS_KEY);

  currentId = hideTips
    ? -1
    : currentId + 1;

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
