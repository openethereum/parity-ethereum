import { BlockNumber, Hash, Integer } from '../types';

export default {
  filter: {
    desc: 'Returns traces matching given filter',
    params: [
      {
        type: Object,
        desc: 'The filter object'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces matching given filter'
    }
  },

  get: {
    desc: 'Returns trace at given position.',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      },
      {
        type: Integer,
        desc: 'Trace position witing transaction'
      }
    ],
    returns: {
      type: Object,
      desc: 'Trace object'
    }
  },

  transaction: {
    desc: 'Returns all traces of given transaction',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces of given transaction'
    }
  },

  block: {
    desc: 'Returns traces created at given block',
    params: [
      {
        type: BlockNumber,
        desc: 'Integer block number, or \'latest\' for the last mined block or \'pending\', \'earliest\' for not yet mined transactions'
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  }
};
