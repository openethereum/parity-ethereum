
import { createAction } from 'redux-actions';

export const error = createAction('error');
export const updateHashrate = createAction('update hashrate');
export const updateBlockNumber = createAction('update blockNumber');
export const updateVersion = createAction('update version');
export const updatePeerCount = createAction('update peerCount');
export const updateNetPeers = createAction('update netPeers');
export const updateNetChain = createAction('update netChain');
export const updateNetPort = createAction('update netPort');
export const updateRpcSettings = createAction('update rpcSettings');
export const updateNodeName = createAction('update nodeName');
export const updateAccounts = createAction('update accounts');
