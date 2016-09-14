// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import sinon from 'sinon';
import RpcProvider from './rpc-provider';

describe('PROVIDER - RPC', () => {
  let cut;

  beforeEach('Mock cut', () => {
    const mockedWeb3Utils = { testUtil: sinon.spy() };
    const mockedWeb3Formatters = { testFormatter: sinon.spy() };
    cut = new RpcProvider(mockedWeb3Utils, mockedWeb3Formatters);
  });

  describe('FORMAT RESULT', () => {
    it('should not format result and coherse to string when no formatter is passed', () => {
      // given
      const result = 5;
      const formatter = null;

      // when
      const returned = cut.formatResult(result, formatter);

      // then
      expect(returned).to.equal('5');
    });

    it('should format with web3Utils and coherse to string when respected formatter is passed', () => {
      // given
      const result = 5;
      const formatter = 'utils.testUtil';

      // when
      cut.formatResult(result, formatter);

      // then
      expect(cut._web3Utils.testUtil.calledWith(result)).to.be.true;
    });

    it('should format with web3Formatters and coherse to string when respected formatter is passed', () => {
      // given
      const result = 5;
      const formatter = 'testFormatter';

      // when
      cut.formatResult(result, formatter);

      // then
      expect(cut._web3Formatters.testFormatter.calledWith(result)).to.be.true;
    });
  });

  describe('FORMAT PARAMS', () => {
    it('should not format params when no formatters are passed', () => {
      // given
      const params = [5, 20];
      const formatters = null;

      // when
      const returned = cut.formatParams(params, formatters);

      // then
      expect(returned).to.eql(params);
    });

    it('should format with web3Utils when respected formatter is passed', () => {
      // given
      const params = [5, 20];
      const formatters = ['utils.testUtil', null];

      // when
      cut.formatParams(params, formatters);

      // then
      expect(cut._web3Utils.testUtil.calledWith(params[0])).to.be.true;
      expect(cut._web3Utils.testUtil.calledOnce).to.be.true;
    });

    it('should format with web3Formatters and coherse to string when respected formatter is passed', () => {
      // given
      const params = [5, 20];
      const formatters = ['testFormatter'];

      // when
      cut.formatParams(params, formatters);

      // then
      expect(cut._web3Formatters.testFormatter.calledWith(params[0])).to.be.true;
      expect(cut._web3Formatters.testFormatter.calledOnce).to.be.true;
    });
  });
});
