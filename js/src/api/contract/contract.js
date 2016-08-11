import Abi from '../../abi';
import Api from '../api';
import { isInstanceOf } from '../util/types';

export default class Contract {
  constructor (eth, abi) {
    if (!isInstanceOf(eth, Api)) {
      throw new Error('EthApi needs to be provided to Contract instance');
    } else if (!abi) {
      throw new Error('Object ABI needs to be provided to Contract instance');
    }

    this._eth = eth;
    this._abi = new Abi(abi);

    this._constructors = this._abi.constructors.map((cons) => this._bindFunction(cons));
    this._functions = this._abi.functions.map((func) => this._bindFunction(func));
    this._events = this._abi.events;
  }

  get address () {
    return this._address;
  }

  get constructors () {
    return this._constructors;
  }

  get events () {
    return this._events;
  }

  get functions () {
    return this._functions;
  }

  get eth () {
    return this._eth;
  }

  get abi () {
    return this._abi;
  }

  at (address) {
    this._address = address;
    return this;
  }

  deploy (code, values, password) {
    const options = {
      data: code,
      gas: 900000
    };

    return this._eth.personal
      .signAndSendTransaction(this._encodeOptions(this.constructors[0], options, values), password)
      .then((txhash) => this.pollTransactionReceipt(txhash))
      .then((receipt) => {
        this._address = receipt.contractAddress;
        return this._eth.eth.getCode(this._address);
      })
      .then((code) => {
        if (code === '0x') {
          throw new Error('Contract not deployed, getCode returned 0x');
        }

        return this.address;
      });
  }

  parseTransactionEvents (receipt) {
    receipt.logs = receipt.logs.map((log) => {
      const signature = log.topics[0].substr(2);
      const event = this.events.find((evt) => evt.signature === signature);

      if (!event) {
        throw new Error(`Unable to find event matching signature ${signature}`);
      }

      const decoded = event.decodeLog(log.topics, log.data);

      log.params = {};
      log.address = decoded.address;

      decoded.params.forEach((param) => {
        log.params[param.name] = param.token.value;
      });

      return log;
    });

    return receipt;
  }

  pollTransactionReceipt (txhash) {
    return new Promise((resolve, reject) => {
      const timeout = () => {
        this._eth.eth
          .getTransactionReceipt(txhash)
          .then((receipt) => {
            if (receipt) {
              resolve(receipt);
            } else {
              setTimeout(timeout, 500);
            }
          })
          .catch((error) => {
            reject(error);
          });
      };

      timeout();
    });
  }

  _encodeOptions (func, options, values) {
    const tokens = this._abi.encodeTokens(func.inputParamTypes(), values);

    if (options.data && options.data.substr(0, 2) === '0x') {
      options.data = options.data.substr(2);
    }

    options.data = `0x${options.data || ''}${func.encodeCall(tokens)}`;

    return options;
  }

  _bindFunction (func) {
    const addAddress = (options) => {
      options.to = options.to || this._address;
      return options;
    };

    func.call = (options, values) => {
      return this._eth.eth
        .call(this._encodeOptions(func, addAddress(options), values))
        .then((encoded) => func.decodeOutput(encoded))
        .then((tokens) => tokens.map((token) => token.value))
        .then((returns) => returns.length === 1 ? returns[0] : returns);
    };

    if (!func.constant) {
      func.sendTransaction = (options, values) => {
        return this._eth.eth
          .sendTransaction(this._encodeOptions(func, addAddress(options), values));
      };

      func.signAndSendTransaction = (options, values, password) => {
        return this._eth.personal
          .signAndSendTransaction(this._encodeOptions(func, addAddress(options), values), password);
      };

      func.estimateGas = (options, values) => {
        return this._eth.eth
          .estimateGas(this._encodeOptions(func, addAddress(options), values));
      };
    }

    return func;
  }
}
