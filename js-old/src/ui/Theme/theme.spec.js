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

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const muiTheme = getMuiTheme(lightBaseTheme);

import theme from './theme';

describe('ui/Theme', () => {
  it('is MUI-based', () => {
    expect(Object.keys(theme)).to.deep.equal(Object.keys(muiTheme).concat('parity'));
  });

  it('allows setting of Parity backgrounds', () => {
    expect(typeof theme.parity.setBackgroundSeed === 'function').to.be.true;
    expect(typeof theme.parity.getBackgroundStyle === 'function').to.be.true;
  });

  describe('parity', () => {
    describe('setBackgroundSeed', () => {
      const SEED = 'testseed';

      beforeEach(() => {
        theme.parity.setBackgroundSeed(SEED);
      });

      it('sets the correct theme values', () => {
        expect(theme.parity.backgroundSeed).to.equal(SEED);
      });
    });

    describe('getBackgroundStyle', () => {
      it('generates a style containing background', () => {
        const style = theme.parity.getBackgroundStyle();

        expect(style).to.have.property('background');
      });
    });
  });
});
