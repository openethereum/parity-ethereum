import { Data } from '../types';

export default {
  getHex: {
    desc: 'Returns binary data from the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      }
    ],
    returns: {
      type: Data,
      desc: 'The previously stored data'
    },
    deprecated: true
  },

  getString: {
    desc: 'Returns string from the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      }
    ],
    returns: {
      type: String,
      desc: 'The previously stored string'
    },
    deprecated: true
  },

  putHex: {
    desc: 'Stores binary data in the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      },
      {
        type: Data,
        desc: 'The data to store'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the value was stored, otherwise `false`'
    },
    deprecated: true
  },

  putString: {
    desc: 'Stores a string in the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      },
      {
        type: String,
        desc: 'The string to store'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the value was stored, otherwise `false`'
    },
    deprecated: true
  }
};
