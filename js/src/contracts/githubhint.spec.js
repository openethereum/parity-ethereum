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

import sinon from 'sinon';

import GithubHint from './githubhint';

let entries;
let githubHint;
let registry;

function create () {
  entries = {
    call: sinon.stub().resolves('testValue')
  };
  registry = {
    getContract: sinon.stub().resolves({ instance: { entries } })
  };
  githubHint = new GithubHint({}, registry);

  return githubHint;
}

describe('contracts/GithubHint', () => {
  beforeEach(() => {
    create();
  });

  it('retrieves the contract via registry', () => {
    expect(registry.getContract).to.have.been.calledWith('githubhint');
  });

  describe('interface', () => {
    describe('getEntry', () => {
      beforeEach(() => {
        return githubHint.getEntry('testId');
      });

      it('calls entries on the instance', () => {
        expect(entries.call).to.have.been.calledWith({}, ['testId']);
      });
    });
  });
});
