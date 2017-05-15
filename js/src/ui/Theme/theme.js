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

import GeoPattern from 'geopattern';
import getMuiTheme from 'material-ui/styles/getMuiTheme';

const muiTheme = getMuiTheme();

muiTheme.zIndex.layer = 4000;
muiTheme.zIndex.popover = 4100;

const imageCache = {};

muiTheme.parity = {
  backgroundSeed: '0x0',

  setBackgroundSeed: (seed) => {
    muiTheme.parity.backgroundSeed = seed;
  },

  getBackgroundStyle: (_gradient, _seed) => {
    const gradient = _gradient || 'rgba(255, 255, 255, 0.25)';
    const seed = _seed || muiTheme.parity.backgroundSeed;
    let url;

    if (_seed) {
      url = GeoPattern.generate(_seed).toDataUrl();
    } else if (imageCache[seed] && imageCache[seed][gradient]) {
      url = imageCache[seed][gradient];
    } else {
      url = GeoPattern.generate(seed).toDataUrl();
      imageCache[seed] = imageCache[seed] || {};
      imageCache[seed][gradient] = url;
    }

    return {
      background: `linear-gradient(${gradient}, ${gradient}), ${url}`
    };
  }
};

export default muiTheme;
