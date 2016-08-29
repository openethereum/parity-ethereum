const options = {
  method: 'GET',
  headers: {
    'Accept': 'application/json'
  }
};

export function call (module, action, _params, test) {
  const host = test ? 'testnet.etherscan.io' : 'api.etherscan.io';
  let params = '';

  if (_params) {
    Object.keys(_params).map((param) => {
      const value = _params[param];

      params = `${params}&${param}=${value}`;
    });
  }

  return fetch(`http://${host}/api?module=${module}&action=${action}${params}`, options)
    .then((response) => {
      if (response.status !== 200) {
        throw { code: response.status, message: response.statusText }; // eslint-disable-line
      }

      return response.json();
    })
    .then((result) => {
      if (result.message === 'NOTOK') {
        throw { code: -1, message: result.result }; // eslint-disable-line
      }

      return result.result;
    });
}
