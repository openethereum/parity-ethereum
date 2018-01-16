/********************************************************************************
*   Ledger Communication toolkit
*   (c) 2016 Ledger
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

/* eslint-disable */

'use strict';

var LedgerEth = function(comm) {
	this.comm = comm;
}

LedgerEth.splitPath = function(path) {
	var result = [];
	var components = path.split('/');
	components.forEach(function (element, index) {
		var number = parseInt(element, 10);
		if (isNaN(number)) {
			return;
		}
		if ((element.length > 1) && (element[element.length - 1] == "'")) {
			number += 0x80000000;
		}
		result.push(number);
	});
	return result;
}

// callback is function(response, error)
LedgerEth.prototype.getAddress = function(path, callback, boolDisplay, boolChaincode) {
	var splitPath = LedgerEth.splitPath(path);
	var buffer = new Buffer(5 + 1 + splitPath.length * 4);
	buffer[0] = 0xe0;
	buffer[1] = 0x02;
	buffer[2] = (boolDisplay ? 0x01 : 0x00);
	buffer[3] = (boolChaincode ? 0x01 : 0x00);
	buffer[4] = 1 + splitPath.length * 4;
	buffer[5] = splitPath.length;
	splitPath.forEach(function (element, index) {
		buffer.writeUInt32BE(element, 6 + 4 * index);
	});
	var self = this;
	var localCallback = function(response, error) {
		if (typeof error != "undefined") {
			callback(undefined, error);
		}
		else {
			var result = {};
			response = new Buffer(response, 'hex');
			var sw = response.readUInt16BE(response.length - 2);
			if (sw != 0x9000) {
				callback(undefined, "Invalid status " + sw.toString(16));
				return;
			}
			var publicKeyLength = response[0];
			var addressLength = response[1 + publicKeyLength];
			result['publicKey'] = response.slice(1, 1 + publicKeyLength).toString('hex');
			result['address'] = "0x" + response.slice(1 + publicKeyLength + 1, 1 + publicKeyLength + 1 + addressLength).toString('ascii');
			if (boolChaincode) {
				result['chainCode'] = response.slice(1 + publicKeyLength + 1 + addressLength, 1 + publicKeyLength + 1 + addressLength + 32).toString('hex');
			}
			callback(result);
		}
	};
	this.comm.exchange(buffer.toString('hex'), localCallback);
}

// callback is function(response, error)
LedgerEth.prototype.signTransaction = function(path, rawTxHex, callback) {
	var splitPath = LedgerEth.splitPath(path);
	var offset = 0;
	var rawTx = new Buffer(rawTxHex, 'hex');
	var apdus = [];
	while (offset != rawTx.length) {
		var maxChunkSize = (offset == 0 ? (150 - 1 - splitPath.length * 4) : 150)
		var chunkSize = (offset + maxChunkSize > rawTx.length ? rawTx.length - offset : maxChunkSize);
		var buffer = new Buffer(offset == 0 ? 5 + 1 + splitPath.length * 4 + chunkSize : 5 + chunkSize);
		buffer[0] = 0xe0;
		buffer[1] = 0x04;
		buffer[2] = (offset == 0 ? 0x00 : 0x80);
		buffer[3] = 0x00;
		buffer[4] = (offset == 0 ? 1 + splitPath.length * 4 + chunkSize : chunkSize);
		if (offset == 0) {
			buffer[5] = splitPath.length;
			splitPath.forEach(function (element, index) {
				buffer.writeUInt32BE(element, 6 + 4 * index);
			});
			rawTx.copy(buffer, 6 + 4 * splitPath.length, offset, offset + chunkSize);
		}
		else {
			rawTx.copy(buffer, 5, offset, offset + chunkSize);
		}
		apdus.push(buffer.toString('hex'));
		offset += chunkSize;
	}
	var self = this;
	var localCallback = function(response, error) {
		if (typeof error != "undefined") {
			callback(undefined, error);
		}
		else {
			response = new Buffer(response, 'hex');
			var sw = response.readUInt16BE(response.length - 2);
			if (sw != 0x9000) {
				callback(undefined, "Invalid status " + sw.toString(16));
				return;
			}
			if (apdus.length == 0) {
					var result = {};
					result['v'] = response.slice(0, 1).toString('hex');
					result['r'] = response.slice(1, 1 + 32).toString('hex');
					result['s'] = response.slice(1 + 32, 1 + 32 + 32).toString('hex');
					callback(result);
			}
			else {
				self.comm.exchange(apdus.shift(), localCallback);
			}
		}
	};
	self.comm.exchange(apdus.shift(), localCallback);
}

// callback is function(response, error)
LedgerEth.prototype.getAppConfiguration = function(callback) {
	var buffer = new Buffer(5);
	buffer[0] = 0xe0;
	buffer[1] = 0x06;
	buffer[2] = 0x00;
	buffer[3] = 0x00;
	buffer[4] = 0x00;
	var localCallback = function(response, error) {
		if (typeof error != "undefined") {
			callback(undefined, error);
		}
		else {
			response = new Buffer(response, 'hex');
			var result = {};
			var sw = response.readUInt16BE(response.length - 2);
			if (sw != 0x9000) {
				callback(undefined, "Invalid status " + sw.toString(16));
				return;
			}
			result['arbitraryDataEnabled'] = (response[0] & 0x01);
			result['version'] = "" + response[1] + '.' + response[2] + '.' + response[3];
			callback(result);
		}
	};
	this.comm.exchange(buffer.toString('hex'), localCallback);
}

module.exports = LedgerEth;

/* eslint-enable */
