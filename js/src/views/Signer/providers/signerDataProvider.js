import logger from '../utils/logger';
import { updateIsConnected, updateIsNodeRunning } from '../actions/signer';

import { isParityRunning } from '../utils/parity';

export default class SignerDataProvider {
  constructor (store, ws) {
    this.store = store;
    this.ws = ws;
    this.ws.onOpen.push(::this.onWsOpen);
    this.ws.onError.push(::this.onWsError);
    this.ws.onClose.push(::this.onWsError);

    this.checkIfIsRunning();
  }

  checkIfIsRunning () {
    const { isNodeRunning, isLoading, url } = this.store.getState().signer;

    isParityRunning(url).then(isRunning => {
      if (isRunning !== isNodeRunning || isLoading) {
        this.store.dispatch(updateIsNodeRunning(isRunning));
      }

      // call later
      const interval = isRunning ? 5000 : 1000;
      setTimeout(() => this.checkIfIsRunning(), interval);
    });
  }

  onWsOpen () {
    logger.log('[APP Provider] connected');
    this.store.dispatch(updateIsConnected(true));
  }

  onWsError () {
    this.store.dispatch(updateIsConnected(false));
  }
}
