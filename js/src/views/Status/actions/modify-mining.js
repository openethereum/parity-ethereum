
import { createAction } from 'redux-actions';

export const modifyMinGasPrice = createAction('modify minGasPrice');
export const modifyGasFloorTarget = createAction('modify gasFloorTarget');
export const modifyAuthor = createAction('modify author');
export const modifyExtraData = createAction('modify extraData');
export const resetExtraData = createAction('reset extraData');
