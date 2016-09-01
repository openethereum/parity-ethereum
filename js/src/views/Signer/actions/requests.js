import { withToastr } from 'dapps-react-components/src/util/toastr';
import { identity } from 'dapps-react-components/src/util/util';

import { createAction } from 'redux-actions';

// TODO [legacy;todr] Remove
export const updateCompatibilityMode = createAction('update compatibilityMode');

export const updatePendingRequests = createAction('update pendingRequests');
export const startConfirmRequest = createAction('start confirmRequest');
export const successConfirmRequest = createAction('success confirmRequest');
export const errorConfirmRequest = createAction('error confirmRequest', identity,
  withToastr(args => args.err, 'error')
);
export const startRejectRequest = createAction('start rejectRequest');
export const successRejectRequest = createAction('success rejectRequest');
export const errorRejectRequest = createAction('error rejectRequest', identity,
  withToastr(args => args.err, 'error')
);
