import Decoder from '../decoder/decoder';
import Encoder from '../encoder/encoder';
import Param from './param';
import { methodSignature } from '../util/signature';

export default class Func {
  constructor (abi) {
    this._name = abi.name;
    this._constant = !!abi.constant;
    this._inputs = Param.toParams(abi.inputs || []);
    this._outputs = Param.toParams(abi.outputs || []);
    this._signature = methodSignature(this._name, this.inputParamTypes());
  }

  get constant () {
    return this._constant;
  }

  get name () {
    return this._name;
  }

  get inputs () {
    return this._inputs;
  }

  get outputs () {
    return this._outputs;
  }

  get signature () {
    return this._signature;
  }

  inputParamTypes () {
    return this._inputs.map((input) => input.kind);
  }

  outputParamTypes () {
    return this._outputs.map((output) => output.kind);
  }

  encodeCall (tokens) {
    return `${this._signature}${Encoder.encode(tokens)}`;
  }

  decodeOutput (data) {
    return Decoder.decode(this.outputParamTypes(), data);
  }
}
