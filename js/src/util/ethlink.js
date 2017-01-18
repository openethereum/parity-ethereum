// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import base32 from 'base32.js';

const BASE_URL = '.web.ethlink.io';
const ENCODER_OPTS = { type: 'crockford' };

export function encode (token, url) {
  const encoder = new base32.Encoder(ENCODER_OPTS);
  const chars = `${token}+${url}`
    .split('')
    .map((char) => char.charCodeAt(0));
  const encoded = encoder
    .write(chars).finalize() // create the encoded string
    .match(/.{1,63}/g) // split into 63-character chunks, max length is 64 for URLs parts
    .join('.'); // add '.' between URL parts

  return `${encoded}${BASE_URL}`;
}

// TODO: This export is really more a helper along the way of verifying the actual
// encoding (being able to decode test values from the node layer), than meant to
// be used as-is. Should the need arrise to decode URLs as well (instead of just
// producing), it would make sense to further split the output into the token/URL
export function decode (encoded) {
  const decoder = new base32.Decoder(ENCODER_OPTS);
  const sanitized = encoded
    .replace(BASE_URL, '') // remove the BASE URL
    .split('.') // split the string on the '.' (63-char boundaries)
    .join(''); // combine without the '.'

  return decoder.write(sanitized).finalize().toString();
}

export {
  BASE_URL
};
