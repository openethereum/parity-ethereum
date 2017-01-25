// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import fs from 'fs';
import path from 'path';
import interfaces from './';
import { Address, BlockNumber, Data, Hash, Integer, Quantity } from './types';

function verifyType (obj) {
  if (typeof obj !== 'string') {
    expect(obj).to.satisfy(() => {
      return obj.type === Array ||
        obj.type === Boolean ||
        obj.type === Object ||
        obj.type === String ||
        obj.type === Address ||
        obj.type === BlockNumber ||
        obj.type === Data ||
        obj.type === Hash ||
        obj.type === Integer ||
        obj.type === Quantity;
    });
  }
}

// Get a list of JSON-RPC from Rust trait source code
function parseMethodsFromRust (source) {
  // Matching the custom `rpc` attribute with it's doc comment
  const attributePattern = /((?:\s*\/\/\/.*$)*)\s*#\[rpc\(([^)]+)\)]/gm;
  const commentPattern = /\s*\/\/\/\s*/g;
  const separatorPattern = /\s*,\s*/g;
  const assignPattern = /([\S]+)\s*=\s*"([^"]*)"/;
  const ignorePattern = /@(deprecated|unimplemented|alias)\b/i;

  const methods = [];

  source.toString().replace(attributePattern, (match, comment, props) => {
    comment = comment.replace(commentPattern, '\n').trim();

    // Skip deprecated methods
    if (ignorePattern.test(comment)) {
      return match;
    }

    props.split(separatorPattern).forEach((prop) => {
      const [, key, value] = prop.split(assignPattern) || [];

      if (key === 'name' && value != null) {
        methods.push(value);
      }
    });

    return match;
  });

  return methods;
}

// Get a list of all JSON-RPC methods from all defined traits
function getMethodsFromRustTraits () {
  const traitsDir = path.join(__dirname, '../../../rpc/src/v1/traits');

  return [
    'eth',
    'eth_signing', // Rolled into `eth`
    'net',
    'parity',
    'parity_accounts',
    'parity_set',
    'parity_signing', // Rolled into `parity`
    'personal',
    'signer',
    'traces',
    'web3'
  ].map((name) => fs.readFileSync(path.join(traitsDir, `${name}.rs`)))
    .map(parseMethodsFromRust)
    .reduce((a, b) => a.concat(b));
}

const rustMethods = {};

getMethodsFromRustTraits().sort().forEach((method) => {
  const [group, name] = method.split('_');

  rustMethods[group] = rustMethods[group] || {};
  rustMethods[group][name] = true;
});

describe('Rust defined JSON-RPC methods', () => {
  Object.keys(rustMethods).forEach((group) => {
    describe(group, () => {
      Object.keys(rustMethods[group]).forEach((name) => {
        describe(name, () => {
          it('has a defined JS interface', () => {
            expect(interfaces[group][name]).to.exist;
          });
        });
      });
    });
  });
});

describe('jsonrpc/interfaces', () => {
  Object.keys(interfaces).forEach((group) => {
    describe(group, () => {
      Object.keys(interfaces[group]).forEach((name) => {
        const method = interfaces[group][name];

        describe(name, () => {
          if (!method.nodoc) {
            it('is present in Rust codebase', () => {
              expect(rustMethods[group][name]).to.exist;
            });
          }

          it('has the correct interface', () => {
            expect(method.desc).to.be.a('string');
            expect(method.params).to.be.an('array');
            expect(method.returns).to.satisfy((returns) => {
              return typeof returns === 'string' || typeof returns === 'object';
            });

            method.params.forEach(verifyType);
            verifyType(method.returns);
          });
        });
      });
    });
  });
});
