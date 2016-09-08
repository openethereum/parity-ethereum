
import { createAction } from 'redux-actions';

export const error = createAction('error');
export const updateAuthor = createAction('update author');
export const updateMinGasPrice = createAction('update minGasPrice');
export const updateGasFloorTarget = createAction('update gasFloorTarget');
export const updateExtraData = createAction('update extraData');
export const updateDefaultExtraData = createAction('update defaultExtraData');
