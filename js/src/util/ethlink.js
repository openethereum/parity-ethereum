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

// Using base-x since we started with base-58 originally. This one allows the
// very slight advantage of using custom dictionaries (useful in the case of
// base-32 where there are multiples available)
import base32 from 'base32.js';

// base32 alphabet should match the Rust implementation
// https://github.com/andreasots/base32/blob/master/src/base32.rs
// const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
const BASE_URL = 'web.ethlink.io';

export function encode (token, url) {
  const encoder = new base32.Encoder({ type: 'crockford' });
  const chars = `${token}+${url}`.split('').map((char) => char.charCodeAt(0));

  return `${encoder.write(chars).finalize()}.${BASE_URL}`;
}

export function decode (encoded) {
  const decoder = new base32.Decoder({ type: 'crockford' });
  const chars = decoder.write(encoded.replace(`.${BASE_URL}`, '')).finalize();

  return chars.toString();
}

export {
  BASE_URL
};
