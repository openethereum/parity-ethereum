import { createAction } from 'redux-actions';

import { identity } from '../util';
import { withError } from '../../../redux/util';

export const copyToClipboard = createAction('copy toClipboard', identity, withError(identity));
