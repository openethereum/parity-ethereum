export function statusBlockNumber (blockNumber) {
  return {
    type: 'statusBlockNumber',
    blockNumber
  };
}

export function statusCollection (collection) {
  return {
    type: 'statusCollection',
    collection
  };
}

export function statusLogs (logInfo) {
  return {
    type: 'statusLogs',
    logInfo
  };
}

export function toggleStatusLogs (devLogsEnabled) {
  return {
    type: 'toggleStatusLogs',
    devLogsEnabled
  };
}

export function clearStatusLogs () {
  return {
    type: 'clearStatusLogs'
  };
}
