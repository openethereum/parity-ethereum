import logger from '../utils/logger';
import { updateIsConnected, updateIsNodeRunning } from '../actions/signer';

import { isParityRunning } from '../utils/parity';

export default class SignerDataProvider {
  constructor (store, ws) {
    this.store = store;
    this.ws = ws;

    this.checkIfIsRunning();

    this.ws.onOpen.push(this.onWsOpen);
    this.ws.onError.push(this.onWsError);
    this.ws.onClose.push(this.onWsError);
  }

  checkIfIsRunning = () => {
    const { isNodeRunning, isLoading, url } = this.store.getState().signer;
    const nextCheck = () => setTimeout(this.checkIfIsRunning, 1000);

    isParityRunning(url)
      .then((isRunning) => {
        if (isRunning !== isNodeRunning || isLoading) {
          this.store.dispatch(updateIsNodeRunning(isRunning));
        }

        nextCheck();
      });
  }

  onWsOpen = () => {
    logger.log('[Signer Provider] connected');
    this.store.dispatch(updateIsConnected(true));
  }

  onWsError = () => {
    logger.log('[Signer Provider] error');
    this.store.dispatch(updateIsConnected(false));
  }
}
