const ENDPOINT = 'https://cors.shapeshift.io';

module.exports = function(apikey) {
  const call = function(method, options) {
    return fetch(`${ENDPOINT}/${method}`, options)
      .then((response) => {
        if (response.status !== 200) {
          throw { code: response.status, message: response.statusText }; // eslint-disable-line
        }

        return response.json();
      })
      .then((result) => {
        if (result.error) {
          throw { code: -1, message: result.error }; // eslint-disable-line
        }

        return result;
      });
  };

  return {
    ENDPOINT: ENDPOINT,

    get: function(method) {
      return call(method, {
        method: 'GET',
        headers: {
          'Accept': 'application/json'
        }
      });
    },

    post: function(method, data) {
      const params = {
        apiKey: apikey
      };

      Object.keys(data).forEach((key) => {
        params[key] = data[key];
      });

      const json = JSON.stringify(params);

      return call(method, {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
          'Content-Length': json.length
        },
        body: json
      });
    }
  };
};
