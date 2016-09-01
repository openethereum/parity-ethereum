export function newTooltip (newId) {
  return {
    type: 'newTooltip',
    newId
  };
}

export function nextTooltip () {
  return {
    type: 'nextTooltip'
  };
}

export function closeTooltips () {
  return {
    type: 'closeTooltips'
  };
}
