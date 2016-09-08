
import { createAction } from 'redux-actions';
import { identity } from '../util';
import { withToastr } from '../util/toastr';

export const updateLogging = createAction(
  'update logging', identity, withToastr(flag => `logging updated to ${flag}`)
);
