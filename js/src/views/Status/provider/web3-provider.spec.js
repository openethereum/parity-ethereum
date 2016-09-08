import sinon from 'sinon';
import { Web3Provider } from './web3-provider';
import * as StatusActions from '../actions/status';

describe('WEB3 PROVIDER', () => {
  let cut;
  let state;
  let web3;

  beforeEach('mock Web3Provider', () => {
    state = {
      status: {
        noOfErrors: 0
      }
    };
    web3 = {
      eth: {
        getHashrate: sinon.spy(),
        getBlockNumber: sinon.spy(),
        getCoinbase: sinon.spy()
      },
      net: {
        getPeerCount: sinon.spy()
      },
      version: {
        getNode: sinon.spy()
      }
    };

    const ethcoreWeb3 = {
      getMinGasPrice: sinon.spy(),
      getGasFloorTarget: sinon.spy(),
      getExtraData: sinon.spy()
    };

    const store = {
      dispatch: sinon.spy(),
      getState: () => state
    };

    cut = new Web3Provider(web3, ethcoreWeb3, store);
  });

  it('should get action from action type', () => {
    // given
    const action = StatusActions.updatePeerCount(20);

    // then
    expect(cut.actionToStateProp(action)).to.equal('peerCount');
  });

  it('should get this.delay when no errors', () => {
    // given
    state.status.noOfErrors = 0;

    // then
    expect(cut.nextDelay()).to.equal(cut.delay);
  });

  it('should get result higher this.delay when there are errors', () => {
    // given
    state.status.noOfErrors = 10;

    // then
    expect(cut.nextDelay()).to.be.above(cut.delay);
  });

  it('should call only single method when you are disconnected', () => {
    // given
    state.status.disconnected = true;

    // when
    cut.onTick();

    // then
    expect(web3.eth.getBlockNumber.called).to.be.true;

    [web3.eth.getHashrate, web3.eth.getCoinbase, web3.net.getPeerCount]
      .map(method => {
        expect(method.called).to.be.false;
      });
  });
});
