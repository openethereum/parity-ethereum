import { combineReducers } from 'redux';
import { routerReducer } from 'react-router-redux';

import { balancesReducer, personalReducer, statusReducer as nodeStatusReducer } from './providers';

import { errorReducer } from '../ui/Errors';
import { tooltipReducer } from '../ui/Tooltips';

import {
  signer as signerReducer,
  requests as signerRequestsReducer
} from '../views/Signer/reducers';

import {
  status as statusReducer,
  debug as statusDebugReducer,
  logger as statusLoggerReducer,
  mining as statusMiningReducer,
  rpc as statusRpcReducer,
  settings as statusSettingsReducer
} from '../views/Status/reducers';

export default function () {
  return combineReducers({
    errors: errorReducer,
    tooltip: tooltipReducer,
    routing: routerReducer,

    balances: balancesReducer,
    nodeStatus: nodeStatusReducer,
    personal: personalReducer,

    signer: signerReducer,
    signerRequests: signerRequestsReducer,
    status: statusReducer,
    statusSettings: statusSettingsReducer,
    statusMining: statusMiningReducer,
    statusRpc: statusRpcReducer,
    statusLogger: statusLoggerReducer,
    statusDebug: statusDebugReducer
  });
}
