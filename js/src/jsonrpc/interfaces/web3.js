import { Data } from '../types';

export default {
  clientVersion: {
    desc: 'Returns the current client version.',
    params: [],
    returns: {
      type: String,
      desc: 'The current client version'
    }
  },

  sha3: {
    desc: 'Returns Keccak-256 (*not* the standardized SHA3-256) of the given data.',
    params: [
      {
        type: String,
        desc: 'The data to convert into a SHA3 hash'
      }
    ],
    returns: {
      type: Data,
      desc: 'The SHA3 result of the given string'
    }
  }
};
