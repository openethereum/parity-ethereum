import { combineReducers } from 'redux';
import { routerReducer } from 'react-router-redux';

import { balancesReducer, personalReducer, statusReducer as nodeStatusReducer } from './providers';

import { errorReducer } from '../ui/Errors';
import { tooltipReducer } from '../ui/Tooltips';

import {
  signer as signerReducer,
  requests as signerRequestsReducer
} from '../views/Signer/reducers';

export default function () {
  return combineReducers({
    errors: errorReducer,
    tooltip: tooltipReducer,
    routing: routerReducer,

    balances: balancesReducer,
    nodeStatus: nodeStatusReducer,
    personal: personalReducer,

    signer: signerReducer,
    signerRequests: signerRequestsReducer
  });
}
