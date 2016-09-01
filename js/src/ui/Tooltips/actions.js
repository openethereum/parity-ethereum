let currentId = -1;
let maxId = 0;

export function newTooltip (id) {
  maxId = Math.max(id, maxId);

  return {
    type: 'newTooltip',
    maxId,
    currentId
  };
}

export function nextTooltip () {
  currentId++;

  return {
    type: 'nextTooltip',
    currentId
  };
}

export function closeTooltips () {
  currentId = -1;

  return {
    type: 'closeTooltips',
    currentId
  };
}
