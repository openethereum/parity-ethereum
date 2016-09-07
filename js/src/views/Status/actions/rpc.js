
import { createAction } from 'redux-actions';
import { identity } from '../util';
import { withToastr } from '../util/toastr';

export const error = createAction('error rpc', identity,
  withToastr(() => 'error processing rpc call. check console for details', 'error')
);
export const fireRpc = createAction('fire rpc');
export const addRpcReponse = createAction('add rpcResponse');
export const selectRpcMethod = createAction('select rpcMethod');
export const resetRpcPrevCalls = createAction('reset rpcPrevCalls');
