// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

const _QR_SIZES = { 'L': [], 'M': [], 'H': [], 'Q': [] };
const QR_LEVELS = Object.keys(_QR_SIZES);

/* eslint-disable indent,no-multi-spaces */
const QR_SIZES = [
    19,   16,   13,    9,
    34,   28,   22,   16,
    55,   44,   34,   26,
    80,   64,   48,   36,
   108,   86,   62,   46,
   136,  108,   76,   60,
   156,  124,   88,   66,
   194,  154,  110,   86,
   232,  182,  132,  100,
   274,  216,  154,  122,
   324,  254,  180,  140,
   370,  290,  206,  158,
   428,  334,  244,  180,
   461,  365,  261,  197,
   523,  415,  295,  223,
   589,  453,  325,  253,
   647,  507,  367,  283,
   721,  563,  397,  313,
   795,  627,  445,  341,
   861,  669,  485,  385,
   932,  714,  512,  406,
  1006,  782,  568,  442,
  1094,  860,  614,  464,
  1174,  914,  664,  514,
  1276, 1000,  718,  538,
  1370, 1062,  754,  596,
  1468, 1128,  808,  628,
  1531, 1193,  871,  661,
  1631, 1267,  911,  701,
  1735, 1373,  985,  745,
  1843, 1455, 1033,  793,
  1955, 1541, 1115,  845,
  2071, 1631, 1171,  901,
  2191, 1725, 1231,  961,
  2306, 1812, 1286,  986,
  2434, 1914, 1354, 1054,
  2566, 1992, 1426, 1096,
  2702, 2102, 1502, 1142,
  2812, 2216, 1582, 1222,
  2956, 2334, 1666, 1276
].reduce((sizes, value, index) => {
  sizes[QR_LEVELS[index % 4]].push(value);

  return sizes;
}, _QR_SIZES);
/* eslint-enable indent,no-multi-spaces */

export function calculateType (lengthBytes, errorLevel = 'M') {
  let type = 5;

  // subtract 3 from the capacities, since we need 2 bits for the mode and a
  // bunch more for the length.
  while (type < 40 && lengthBytes > QR_SIZES[errorLevel][type - 1] - 3) {
    type++;
  }

  return type;
}
