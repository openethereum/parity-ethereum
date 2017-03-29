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

import { numberFromString } from './numberFromString';

describe('views/Status/components/MiningSettings/numberFromString', () => {
  it('should convert string to number', () => {
    expect(numberFromString('12345'), 12345);
  });

  it('should handle special characters "k" and "m"', () => {
    expect(numberFromString('10kk'), 10000000);
    expect(numberFromString('10K'), 1000);
    expect(numberFromString('10Mmk'), 1000000000000000);
  });

  it('should ignore any non-numeric characters', () => {
    expect(numberFromString('10.000.000'), 10000000);
    expect(numberFromString('10_000_000'), 10000000);
    expect(numberFromString('10_k_k'), 10000000);
    expect(numberFromString('-5'), 5);
  });
});
