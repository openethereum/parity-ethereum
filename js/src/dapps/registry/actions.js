import { createAction } from 'redux-actions';

export default {
  fetchContract: createAction('fetch contract'),
  setContract: createAction('set contract', (c) => c),

  fetchFee: createAction('fetch fee'),
  setFee: createAction('set fee', (f) => f),

  fetchOwner: createAction('fetch owner'),
  setOwner: createAction('set owner', (f) => f)
};
