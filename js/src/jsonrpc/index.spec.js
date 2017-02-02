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

import fs from 'fs';
import path from 'path';
import interfaces from './';
import * as customTypes from './types';

const allowedTypes = [Array, Boolean, Object, String].concat(Object.values(customTypes));

function verifyType (obj) {
  if (typeof obj !== 'string') {
    expect(obj).to.satisfy(() => allowedTypes.includes(obj.type));
  }
}

// Get a list of JSON-RPC from Rust trait source code
function parseMethodsFromRust (source) {
  // Matching the custom `rpc` attribute with it's doc comment
  const attributePattern = /((?:\s*\/\/\/.*$)*)\s*#\[rpc\(([^)]+)\)]/gm;
  const commentPattern = /\s*\/\/\/\s*/g;
  const separatorPattern = /\s*,\s*/g;
  const assignPattern = /([\S]+)\s*=\s*"([^"]*)"/;
  const ignorePattern = /@(ignore|deprecated|unimplemented|alias)\b/i;

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

  return fs.readdirSync(traitsDir)
            .filter((name) => name !== 'mod.rs' && /\.rs$/.test(name))
            .map((name) => fs.readFileSync(path.join(traitsDir, name)))
            .map(parseMethodsFromRust)
            .reduce((a, b) => a.concat(b));
}

const rustMethods = {};

getMethodsFromRustTraits().sort().forEach((method) => {
  const [group, name] = method.split('_');

  // Skip methods with malformed names
  if (group == null || name == null) {
    return;
  }

  rustMethods[group] = rustMethods[group] || {};
  rustMethods[group][name] = true;
});

describe('jsonrpc/interfaces', () => {
  describe('Rust trait methods', () => {
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
