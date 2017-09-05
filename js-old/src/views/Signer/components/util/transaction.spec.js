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

import BigNumber from 'bignumber.js';
import { getShortData, getFee, getTotalValue } from './transaction';

describe('views/Signer/components/util/transaction', () => {
  describe('getEstimatedMiningTime', () => {
    it('should return estimated mining time', () => {
    });
  });

  describe('getShortData', () => {
    it('should return short data', () => {
      // given
      const data = '0xh87dY78';

      // when
      const res = getShortData(data);

      // then
      expect(res).to.equal('0xh...');
    });

    it('should return data as is', () => {
      // given
      const data = '0x0';

      // when
      const shortData = getShortData(data);

      // then
      expect(shortData).to.equal('0x0');
    });
  });

  describe('getFee', () => {
    it('should return wei BigNumber object equals to gas * gasPrice', () => {
      // given
      const gas = '0x76c0'; // 30400
      const gasPrice = '0x9184e72a000'; // 10000000000000 wei

      // when
      const fee = getFee(gas, gasPrice);

      // then
      expect(fee).to.be.an.instanceOf(BigNumber);
      expect(fee.toString()).to.be.equal('304000000000000000'); // converting to string due to https://github.com/MikeMcl/bignumber.js/issues/11
    });
  });

  describe('getTotalValue', () => {
    it('should return wei BigNumber totalValue equals to value + fee', () => {
      // given
      const fee = new BigNumber(304000000000000000); // wei
      const value = '0x9184e72a'; // 2441406250 wei

      // when
      const totalValue = getTotalValue(fee, value);

      // then
      expect(totalValue).to.be.an.instanceOf(BigNumber);
      expect(totalValue.toString()).to.be.equal('304000002441406250'); // converting to string due to https://github.com/MikeMcl/bignumber.js/issues/11
    });
  });
});
