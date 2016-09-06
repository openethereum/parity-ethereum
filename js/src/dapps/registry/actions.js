import { createAction } from 'redux-actions';

export const fetchContract = createAction('fetch contract');
export const setContract = createAction('set contract', (c) => c);

export const fetchFee = createAction('fetch fee');
export const setFee = createAction('set fee', (f) => f);

export const fetchOwner = createAction('fetch owner');
export const setOwner = createAction('set owner', (f) => f);
