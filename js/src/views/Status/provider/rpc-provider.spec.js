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
