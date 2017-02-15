// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import BigNumber from 'bignumber.js';

import { toChecksumAddress } from '../../abi/util/address';
import { isString } from '../util/types';

export function outAccountInfo (infos) {
  return Object
    .keys(infos)
    .reduce((ret, _address) => {
      const info = infos[_address];
      const address = outAddress(_address);

      ret[address] = {
        name: info.name
      };

      if (info.meta) {
        ret[address].uuid = info.uuid;
        ret[address].meta = JSON.parse(info.meta);
      }

      return ret;
    }, {});
}

export function outAddress (address) {
  return toChecksumAddress(address);
}

export function outAddresses (addresses) {
  return (addresses || []).map(outAddress);
}

export function outBlock (block) {
  if (block) {
    Object.keys(block).forEach((key) => {
      switch (key) {
        case 'author':
        case 'miner':
          block[key] = outAddress(block[key]);
          break;

        case 'difficulty':
        case 'gasLimit':
        case 'gasUsed':
        case 'nonce':
        case 'number':
        case 'totalDifficulty':
          block[key] = outNumber(block[key]);
          break;

        case 'timestamp':
          block[key] = outDate(block[key]);
          break;
      }
    });
  }

  return block;
}

export function outChainStatus (status) {
  if (status) {
    Object.keys(status).forEach((key) => {
      switch (key) {
        case 'blockGap':
          status[key] = status[key]
            ? status[key].map(outNumber)
            : status[key];
          break;
      }
    });
  }

  return status;
}

export function outDate (date) {
  return new Date(outNumber(date).toNumber() * 1000);
}

export function outHistogram (histogram) {
  if (histogram) {
    Object.keys(histogram).forEach((key) => {
      switch (key) {
        case 'bucketBounds':
        case 'counts':
          histogram[key] = histogram[key].map(outNumber);
          break;
      }
    });
  }

  return histogram;
}

export function outLog (log) {
  Object.keys(log).forEach((key) => {
    switch (key) {
      case 'blockNumber':
      case 'logIndex':
      case 'transactionIndex':
        log[key] = outNumber(log[key]);
        break;

      case 'address':
        log[key] = outAddress(log[key]);
        break;
    }
  });

  return log;
}

export function outHwAccountInfo (infos) {
  return Object
    .keys(infos)
    .reduce((ret, _address) => {
      const address = outAddress(_address);

      ret[address] = infos[_address];

      return ret;
    }, {});
}

export function outNumber (number) {
  return new BigNumber(number || 0);
}

export function outPeer (peer) {
  const protocols = Object.keys(peer.protocols)
    .reduce((obj, key) => {
      if (peer.protocols[key]) {
        obj[key] = {
          ...peer.protocols[key],
          difficulty: outNumber(peer.protocols[key].difficulty)
        };
      }

      return obj;
    }, {});

  return {
    ...peer,
    protocols
  };
}

export function outPeers (peers) {
  return {
    active: outNumber(peers.active),
    connected: outNumber(peers.connected),
    max: outNumber(peers.max),
    peers: peers.peers.map((peer) => outPeer(peer))
  };
}

export function outReceipt (receipt) {
  if (receipt) {
    Object.keys(receipt).forEach((key) => {
      switch (key) {
        case 'blockNumber':
        case 'cumulativeGasUsed':
        case 'gasUsed':
        case 'transactionIndex':
          receipt[key] = outNumber(receipt[key]);
          break;

        case 'contractAddress':
          receipt[key] = outAddress(receipt[key]);
          break;
      }
    });
  }

  return receipt;
}

export function outRecentDapps (recentDapps) {
  if (recentDapps) {
    Object.keys(recentDapps).forEach((url) => {
      recentDapps[url] = outDate(recentDapps[url]);
    });
  }

  return recentDapps;
}

export function outSignerRequest (request) {
  if (request) {
    Object.keys(request).forEach((key) => {
      switch (key) {
        case 'id':
          request[key] = outNumber(request[key]);
          break;

        case 'payload':
          request[key].signTransaction = outTransaction(request[key].signTransaction);
          request[key].sendTransaction = outTransaction(request[key].sendTransaction);
          break;

        case 'origin':
          const type = Object.keys(request[key])[0];
          const details = request[key][type];

          request[key] = { type, details };
          break;
      }
    });
  }

  return request;
}

export function outSyncing (syncing) {
  if (syncing && syncing !== 'false') {
    Object.keys(syncing).forEach((key) => {
      switch (key) {
        case 'currentBlock':
        case 'highestBlock':
        case 'startingBlock':
        case 'warpChunksAmount':
        case 'warpChunksProcessed':
          syncing[key] = outNumber(syncing[key]);
          break;

        case 'blockGap':
          syncing[key] = syncing[key] ? syncing[key].map(outNumber) : syncing[key];
          break;
      }
    });
  }

  return syncing;
}

export function outTransactionCondition (condition) {
  if (condition) {
    if (condition.block) {
      condition.block = outNumber(condition.block);
    } else if (condition.time) {
      condition.time = outDate(condition.time);
    }
  }

  return condition;
}

export function outTransaction (tx) {
  if (tx) {
    Object.keys(tx).forEach((key) => {
      switch (key) {
        case 'blockNumber':
        case 'gasPrice':
        case 'gas':
        case 'nonce':
        case 'transactionIndex':
        case 'value':
          tx[key] = outNumber(tx[key]);
          break;

        case 'condition':
          tx[key] = outTransactionCondition(tx[key]);
          break;

        case 'minBlock':
          tx[key] = tx[key]
            ? outNumber(tx[key])
            : null;
          break;

        case 'creates':
        case 'from':
        case 'to':
          tx[key] = outAddress(tx[key]);
          break;
      }
    });
  }

  return tx;
}

export function outTrace (trace) {
  if (trace) {
    if (trace.action) {
      Object.keys(trace.action).forEach(key => {
        switch (key) {
          case 'gas':
          case 'value':
          case 'balance':
            trace.action[key] = outNumber(trace.action[key]);
            break;

          case 'from':
          case 'to':
          case 'address':
          case 'refundAddress':
            trace.action[key] = outAddress(trace.action[key]);
            break;
        }
      });
    }

    if (trace.result) {
      Object.keys(trace.result).forEach(key => {
        switch (key) {
          case 'gasUsed':
            trace.result[key] = outNumber(trace.result[key]);
            break;

          case 'address':
            trace.action[key] = outAddress(trace.action[key]);
            break;
        }
      });
    }

    if (trace.traceAddress) {
      trace.traceAddress.forEach((address, index) => {
        trace.traceAddress[index] = outNumber(address);
      });
    }

    Object.keys(trace).forEach((key) => {
      switch (key) {
        case 'subtraces':
        case 'transactionPosition':
        case 'blockNumber':
          trace[key] = outNumber(trace[key]);
          break;
      }
    });
  }

  return trace;
}

export function outTraces (traces) {
  if (traces) {
    return traces.map(outTrace);
  }

  return traces;
}

export function outTraceReplay (trace) {
  if (trace) {
    Object.keys(trace).forEach((key) => {
      switch (key) {
        case 'trace':
          trace[key] = outTraces(trace[key]);
          break;
      }
    });
  }

  return trace;
}

export function outVaultMeta (meta) {
  if (isString(meta)) {
    try {
      const obj = JSON.parse(meta);

      return obj;
    } catch (error) {
      return {};
    }
  }

  return meta || {};
}
