import { inAddress, inData, inNumber16 } from '../../format/input';
import { outAddress, outNumber, outPeers } from '../../format/output';

export default class Ethcore {
  constructor (transport) {
    this._transport = transport;
  }

  acceptNonReservedPeers () {
    return this._transport
      .execute('ethcore_acceptNonReservedPeers');
  }

  addReservedPeer (encode) {
    return this._transport
      .execute('ethcore_addReservedPeer', encode);
  }

  defaultExtraData () {
    return this._transport
      .execute('ethcore_defaultExtraData');
  }

  devLogs () {
    return this._transport
      .execute('ethcore_devLogs');
  }

  devLogsLevels () {
    return this._transport
      .execute('ethcore_devLogsLevels');
  }

  dropNonReservedPeers () {
    return this._transport
      .execute('ethcore_dropNonReservedPeers');
  }

  extraData () {
    return this._transport
      .execute('ethcore_extraData');
  }

  gasFloorTarget () {
    return this._transport
      .execute('ethcore_gasFloorTarget')
      .then(outNumber);
  }

  generateSecretPhrase () {
    return this._transport
      .execute('ethcore_generateSecretPhrase');
  }

  minGasPrice () {
    return this._transport
      .execute('ethcore_minGasPrice')
      .then(outNumber);
  }

  netChain () {
    return this._transport
      .execute('ethcore_netChain');
  }

  netPeers () {
    return this._transport
      .execute('ethcore_netPeers')
      .then(outPeers);
  }

  netMaxPeers () {
    return this._transport
      .execute('ethcore_netMaxPeers')
      .then(outNumber);
  }

  netPort () {
    return this._transport
      .execute('ethcore_netPort')
      .then(outNumber);
  }

  nodeName () {
    return this._transport
      .execute('ethcore_nodeName');
  }

  phraseToAddress (phrase) {
    return this._transport
      .execute('ethcore_phraseToAddress', phrase)
      .then(outAddress);
  }

  removeReservedPeer (encode) {
    return this._transport
      .execute('ethcore_removeReservedPeer', encode);
  }

  rpcSettings () {
    return this._transport
      .execute('ethcore_rpcSettings');
  }

  setAuthor (address) {
    return this._transport
      .execute('ethcore_setAuthor', inAddress(address));
  }

  setExtraData (data) {
    return this._transport
      .execute('ethcore_setExtraData', inData(data));
  }

  setGasFloorTarget (quantity) {
    return this._transport
      .execute('ethcore_setGasFloorTarget', inNumber16(quantity));
  }

  setMinGasPrice (quantity) {
    return this._transport
      .execute('ethcore_setMinGasPrice', inNumber16(quantity));
  }

  setTransactionsLimit (quantity) {
    return this._transport
      .execute('ethcore_setTransactionsLimit', inNumber16(quantity));
  }

  transactionsLimit () {
    return this._transport
      .execute('ethcore_transactionsLimit')
      .then(outNumber);
  }

  unsignedTransactionsCount () {
    return this._transport
      .execute('ethcore_unsignedTransactionsCount')
      .then(outNumber);
  }
}
