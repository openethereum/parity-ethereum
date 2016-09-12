import { combineReducers } from 'redux';
import { routerReducer } from 'react-router-redux';

import { personalReducer } from './providers';
import { errorReducer } from '../ui/Errors';
import { tooltipReducer } from '../ui/Tooltips';
import { nodeStatusReducer } from '../views/Application/Status';

import { signer as signerReducer, requests as signerRequestsReducer } from '../views/Signer/reducers';
import { status as statusReducer, debug as statusDebugReducer, logger as statusLoggerReducer, mining as statusMiningReducer, rpc as statusRpcReducer, settings as statusSettingsReducer } from '../views/Status/reducers';

export default function () {
  return combineReducers({
    errors: errorReducer,
    nodeStatus: nodeStatusReducer,
    tooltip: tooltipReducer,
    routing: routerReducer,
    signer: signerReducer,
    signerRequests: signerRequestsReducer,
    status: statusReducer,
    statusSettings: statusSettingsReducer,
    statusMining: statusMiningReducer,
    statusRpc: statusRpcReducer,
    statusLogger: statusLoggerReducer,
    statusDebug: statusDebugReducer,

    personal: personalReducer
  });
}
