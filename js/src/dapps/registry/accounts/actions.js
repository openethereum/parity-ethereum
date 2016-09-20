import { personal } from '../parity.js';

export const select = (address) => ({ type: 'accounts select', address });
