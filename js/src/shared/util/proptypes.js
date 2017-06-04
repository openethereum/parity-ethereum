import PropTypes from 'prop-types';

export function arrayOrObjectProptype () {
  return PropTypes.oneOfType([
    PropTypes.array,
    PropTypes.object
  ]);
}

export function nullableProptype (type) {
  return PropTypes.oneOfType([
    PropTypes.oneOf([ null ]),
    type
  ]);
}

export function nodeOrStringProptype () {
  return PropTypes.oneOfType([
    PropTypes.node,
    PropTypes.string
  ]);
}
