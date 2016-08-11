import { Http, Ws } from './transport/index';
import Contract from './contract/index';

import { Db, Eth, Ethcore, Net, Personal, Shh, Trace, Web3 } from './rpc/index';
import format from './format/index';
import { isFunction } from './util/types';

export default class Api {
  constructor (transport) {
    if (!transport || !isFunction(transport.execute)) {
      throw new Error('EthApi needs transport with execute() function defined');
    }

    this._db = new Db(transport);
    this._eth = new Eth(transport);
    this._ethcore = new Ethcore(transport);
    this._net = new Net(transport);
    this._personal = new Personal(transport);
    this._shh = new Shh(transport);
    this._trace = new Trace(transport);
    this._web3 = new Web3(transport);
  }

  get db () {
    return this._db;
  }

  get eth () {
    return this._eth;
  }

  get ethcore () {
    return this._ethcore;
  }

  get net () {
    return this._net;
  }

  get personal () {
    return this._personal;
  }

  get shh () {
    return this._shh;
  }

  get trace () {
    return this._trace;
  }

  get web3 () {
    return this._web3;
  }

  static format = format;

  static Contract = Contract

  static Transport = {
    Http: Http,
    Ws: Ws
  }
}
