import ParamType from './paramType';

export function toParamType (type) {
  if (type[type.length - 1] === ']') {
    const last = type.lastIndexOf('[');
    const length = type.substr(last + 1, type.length - last - 2);
    const subtype = toParamType(type.substr(0, last));

    if (length.length === 0) {
      return new ParamType('array', subtype);
    }

    return new ParamType('fixedArray', subtype, parseInt(length, 10));
  }

  switch (type) {
    case 'address':
    case 'bool':
    case 'bytes':
    case 'string':
      return new ParamType(type);

    case 'int':
    case 'uint':
      return new ParamType(type, null, 256);

    default:
      if (type.indexOf('uint') === 0) {
        return new ParamType('uint', null, parseInt(type.substr(4), 10));
      } else if (type.indexOf('int') === 0) {
        return new ParamType('int', null, parseInt(type.substr(3), 10));
      } else if (type.indexOf('bytes') === 0) {
        return new ParamType('fixedBytes', null, parseInt(type.substr(5), 10));
      }

      throw new Error(`Cannot convert ${type} to valid ParamType`);
  }
}

export function fromParamType (paramType) {
  switch (paramType.type) {
    case 'address':
    case 'bool':
    case 'bytes':
    case 'string':
      return paramType.type;

    case 'int':
    case 'uint':
      return `${paramType.type}${paramType.length}`;

    case 'fixedBytes':
      return `bytes${paramType.length}`;

    case 'fixedArray':
      return `${fromParamType(paramType.subtype)}[${paramType.length}]`;

    case 'array':
      return `${fromParamType(paramType.subtype)}[]`;

    default:
      throw new Error(`Cannot convert from ParamType ${paramType.type}`);
  }
}
