import Encoder from '../encoder/encoder';
import Param from './param';

export default class Constructor {
  constructor (abi) {
    this._inputs = Param.toParams(abi.inputs || []);
  }

  get inputs () {
    return this._inputs;
  }

  inputParamTypes () {
    return this._inputs.map((input) => input.kind);
  }

  encodeCall (tokens) {
    return Encoder.encode(tokens);
  }
}
