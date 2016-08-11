import ParamType from '../paramType/paramType';
import Token from '../../token/token';
import { isInstanceOf } from '../../util/types';

export default class DecodedLogParam {
  constructor (name, kind, token) {
    if (!isInstanceOf(kind, ParamType)) {
      throw new Error('kind not instanceof ParamType');
    } else if (!isInstanceOf(token, Token)) {
      throw new Error('token not instanceof Token');
    }

    this._name = name;
    this._kind = kind;
    this._token = token;
  }

  get name () {
    return this._name;
  }

  get kind () {
    return this._kind;
  }

  get token () {
    return this._token;
  }
}
