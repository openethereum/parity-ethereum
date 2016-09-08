import { createAction } from 'redux-actions';

import { identity } from '../util';
import { withError } from '../../../middleware';

export const updateLogging = createAction(
  'update logging', identity, withError(flag => `logging updated to ${flag}`)
);
