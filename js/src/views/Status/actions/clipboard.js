
import { createAction } from 'redux-actions';
import { identity } from '../util';
import { withToastr } from '../util/toastr';

export const copyToClipboard = createAction('copy toClipboard', identity, withToastr(identity));
