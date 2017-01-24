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
  // Matching the custom `rpc` attribute
  const regex = /#\[rpc\((?:async\s*,\s*)?name\s*=\s*"([^"]+)"\)]/g;

  const methods = [];

  source.toString().replace(regex, (match, method) => {
    methods.push(method);

    return match;
  });

  return methods;
}

// Get a list of all JSON-RPC methods from all defined traits
function getMethodsFromRustTraits () {
  const traitsDir = path.join(__dirname, '../../../rpc/src/v1/traits');

  return fs.readdirSync(traitsDir)
    .filter((name) => (name !== 'mod.rs' && /\.rs$/.test(name)))
    .map((name) => fs.readFileSync(path.join(traitsDir, name)))
    .map(parseMethodsFromRust)
    .reduce((a, b) => a.concat(b));
}

const rustMethods = {};

getMethodsFromRustTraits().forEach((method) => {
  const [group, name] = method.split('_');

  rustMethods[group] = rustMethods[group] || {};
  rustMethods[group][name] = true;
});

describe('Rust defined JSON-RPC methods', () => {
  Object.keys(rustMethods).forEach((group) => {
    describe(group, () => {
      Object.keys(rustMethods[group]).forEach((name) => {
        describe(`${group}_${name}`, () => {
          it('has a defined JS interface', () => {
            expect(rustMethods[group][name]).to.be.true;
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

        describe(`${group}_${name}`, () => {
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
