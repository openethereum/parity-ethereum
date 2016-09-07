
import { stringifyIfObject } from '../util';

export default class RpcProvider {

  constructor (web3Utils, web3Formatters) {
    this._web3Utils = web3Utils;
    this._web3Formatters = web3Formatters;
  }

  formatResult (result, formatterName) {
    if (!formatterName) {
      return typeof result === 'object' ? result : String(result);
    }

    let formatter;

    if (formatterName.indexOf('utils.') > -1) {
      formatter = this._web3Utils[formatterName.split('.')[1]];
    } else {
      formatter = this._web3Formatters[formatterName];
    }

    try {
      return `${formatter(result)}`;
    } catch (err) {
      result = stringifyIfObject(result);
      const msg = `error using ${formatterName} on ${result}: ${err}`;
      console.error(msg);
      return new Error(msg);
    }
  }

  formatParams (params, inputFormatters) {
    if (!inputFormatters || !inputFormatters.length) {
      return params;
    }

    return params.map((param, i) => {
      let formatterName = inputFormatters[i];
      if (!formatterName) {
        return param;
      }

      let formatter;

      if (formatterName.indexOf('utils.') > -1) {
        formatter = this._web3Utils[formatterName.split('.')[1]];
      } else {
        formatter = this._web3Formatters[formatterName];
      }

      try {
        return formatter(param);
      } catch (err) {
        param = stringifyIfObject(param);
        const msg = `error using ${formatterName} on ${param}: ${err}`;
        console.error(msg);
        return new Error(msg);
      }
    });
  }
}
