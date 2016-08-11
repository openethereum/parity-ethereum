const TYPES = ['raw', 'prefixed', 'fixedArray', 'array'];

import { padU32 } from '../util/pad';

export default class Mediate {
  constructor (type, value) {
    Mediate.validateType(type);

    this._type = type;
    this._value = value;
  }

  initLength () {
    switch (this._type) {
      case 'raw':
        return this._value.length / 2;

      case 'array':
      case 'prefixed':
        return 32;

      case 'fixedArray':
        return this._value
          .reduce((total, mediate) => {
            return total + mediate.initLength();
          }, 0);
    }
  }

  closingLength () {
    switch (this._type) {
      case 'raw':
        return 0;

      case 'prefixed':
        return this._value.length / 2;

      case 'array':
        return this._value
          .reduce((total, mediate) => {
            return total + mediate.initLength();
          }, 32);

      case 'fixedArray':
        return this._value
          .reduce((total, mediate) => {
            return total + mediate.initLength() + mediate.closingLength();
          }, 0);
    }
  }

  init (suffixOffset) {
    switch (this._type) {
      case 'raw':
        return this._value;

      case 'fixedArray':
        return this._value
          .map((mediate, idx) => mediate.init(Mediate.offsetFor(this._value, idx)).toString(16))
          .join('');

      case 'prefixed':
      case 'array':
        return padU32(suffixOffset);
    }
  }

  closing (offset) {
    switch (this._type) {
      case 'raw':
        return '';

      case 'prefixed':
        return this._value;

      case 'fixedArray':
        return this._value
          .map((mediate, idx) => mediate.closing(Mediate.offsetFor(this._value, idx)).toString(16))
          .join('');

      case 'array':
        const prefix = padU32(this._value.length);
        const inits = this._value
          .map((mediate, idx) => mediate.init(offset + Mediate.offsetFor(this._value, idx) + 32).toString(16))
          .join('');
        const closings = this._value
          .map((mediate, idx) => mediate.closing(offset + Mediate.offsetFor(this._value, idx)).toString(16))
          .join('');

        return `${prefix}${inits}${closings}`;
    }
  }

  get type () {
    return this._type;
  }

  get value () {
    return this._value;
  }

  static offsetFor (mediates, position) {
    if (position < 0 || position >= mediates.length) {
      throw new Error(`Invalid position ${position} specified for Mediate.offsetFor`);
    }

    const initLength = mediates
      .reduce((total, mediate) => {
        return total + mediate.initLength();
      }, 0);

    return mediates
      .slice(0, position)
      .reduce((total, mediate) => {
        return total + mediate.closingLength();
      }, initLength);
  }

  static validateType (type) {
    if (TYPES.filter((_type) => type === _type).length) {
      return true;
    }

    throw new Error(`Invalid type ${type} received for Mediate.validateType`);
  }
}
