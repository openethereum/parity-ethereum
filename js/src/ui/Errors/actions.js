export function newError (error) {
  return {
    type: 'newError',
    error
  };
}

export function closeErrors () {
  return {
    type: 'closeErrors'
  };
}
