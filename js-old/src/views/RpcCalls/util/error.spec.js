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

import sinon from 'sinon';
import * as ErrorUtil from './error';

describe('views/Status/util/error', () => {
  beforeEach('spy on isError', () => {
    sinon.spy(ErrorUtil, 'isError');
  });

  afterEach('spy on isError', () => {
    ErrorUtil.isError.restore();
  });

  describe('filterErrors', () => {
    const ERROR1 = new Error('abc');
    const ERROR2 = new Error('def');
    const INPUT = [ERROR1, 'ghi', ERROR2, 'jkl'];
    const ERRORS = [ERROR1, ERROR2];

    it('should return errors in the array', () => {
      expect(ErrorUtil.filterErrors(INPUT)).to.deep.equal(ERRORS);
    });
  });

  describe('hasErrors', () => {
    it('should return undefined and not invoke isError when null is passed', () => {
      // given
      const xs = null;

      // when
      const res = ErrorUtil.hasErrors(xs);

      // then
      expect(ErrorUtil.isError.called).to.be.false;
      expect(res).to.be.undefined;
    });

    it('should return true and invoke isError when at least one error object is passed', () => {
      // given
      const arg1 = 'test string';
      const arg2 = new Error();
      const xs = [arg1, arg2];

      // when
      const res = ErrorUtil.hasErrors(xs);

      // then
      // todo [adgo] - 30.04.2016 - fix and uncomment
      // expect(ErrorUtil.isError.calledWith(arg1)).to.be.true;
      // expect(ErrorUtil.isError.calledWith(arg2)).to.be.true;
      expect(res).to.be.true;
    });

    it('should return false and invoke isError when non error objects are passed', () => {
      // given
      const arg1 = 'test string';
      const arg2 = 123;
      const xs = [arg1, arg2];

      // when
      const res = ErrorUtil.hasErrors(xs);

      // then
      // todo [adgo] - 30.04.2016 - fix and uncomment
      // expect(ErrorUtil.isError.calledWith(arg1)).to.be.true;
      // expect(ErrorUtil.isError.calledWith(arg2)).to.be.true;
      expect(res).to.be.false;
    });
  });

  describe('isError', () => {
    it('should return false when non error object is passed', () => {
      // given
      const arg = '';

      // when
      const res = ErrorUtil.isError(arg);

      // then
      expect(res).to.be.false;
    });

    it('should return true when error object is passed', () => {
      // given
      const arg = new Error();

      // when
      const res = ErrorUtil.isError(arg);

      // then
      expect(res).to.be.true;
    });
  });
});
