export function isArray (test) {
  return Object.prototype.toString.call(test) === '[object Array]';
}

export function isString (test) {
  return Object.prototype.toString.call(test) === '[object String]';
}

export function isInstanceOf (test, clazz) {
  return test instanceof clazz;
}
