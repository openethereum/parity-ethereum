import JsonRpcBase from '../jsonRpcBase';

/* global fetch */
export default class Http extends JsonRpcBase {
  constructor (url) {
    super();

    this._url = url;
  }

  _encodeOptions (method, params) {
    const json = this.encode(method, params);

    this.log(json);

    return {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'Content-Length': json.length
      },
      body: json
    };
  }

  execute (method, ...params) {
    return fetch(this._url, this._encodeOptions(method, params))
      .then((response) => {
        if (response.status !== 200) {
          this.error(JSON.stringify({ status: response.status, statusText: response.statusText }));
          throw new Error(`${response.status}: ${response.statusText}`);
        }

        return response.json();
      })
      .then((result) => {
        if (result.error) {
          this.error(JSON.stringify(result));
          throw new Error(`${result.error.code}: ${result.error.message}`);
        }

        this.log(JSON.stringify(result));
        return result.result;
      });
  }
}
