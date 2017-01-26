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

const COLORS = {
  default: 'rgba(255, 99, 132, 0.2)',
  selected: 'rgba(255, 99, 132, 0.5)',
  hover: 'rgba(255, 99, 132, 0.15)',
  grid: 'rgba(255, 99, 132, 0.5)',
  line: 'rgb(255, 99, 132)',
  intersection: '#fff'
};

const countModifier = (count) => {
  const val = count.toNumber ? count.toNumber() : count;

  return Math.log10(val + 1) + 0.1;
};

export {
  COLORS,
  countModifier
};
