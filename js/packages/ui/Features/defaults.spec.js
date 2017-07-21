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

import defaults, { FEATURES, MODES } from './defaults';

const features = Object.values(FEATURES);
const modes = Object.values(MODES);

describe('ui/Features/Defaults', () => {
  describe('feature codes', () => {
    Object.keys(FEATURES).forEach((key) => {
      describe(key, () => {
        let value;

        beforeEach(() => {
          value = FEATURES[key];
        });

        it('exists as an default', () => {
          expect(defaults[value]).to.be.ok;
        });

        it('has a single unique code', () => {
          expect(features.filter((code) => code === value).length).to.equal(1);
        });
      });
    });
  });

  describe('defaults', () => {
    Object.keys(defaults).forEach((key) => {
      describe(key, () => {
        let value;

        beforeEach(() => {
          value = defaults[key];
        });

        it('exists as an exposed feature', () => {
          expect(features.includes(key)).to.be.ok;
        });

        it('has a valid mode', () => {
          expect(modes.includes(value.mode)).to.be.true;
        });

        it('has a name and description', () => {
          expect(value.description).to.be.ok;
          expect(value.name).to.be.ok;
        });
      });
    });
  });
});
