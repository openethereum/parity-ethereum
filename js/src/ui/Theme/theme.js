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
import darkBaseTheme from 'material-ui/styles/baseThemes/darkBaseTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const lightTheme = getMuiTheme(lightBaseTheme);
const muiTheme = getMuiTheme(darkBaseTheme);

muiTheme.inkBar.backgroundColor = 'transparent';
muiTheme.paper.backgroundColor = 'rgb(18, 18, 18)';
muiTheme.raisedButton.primaryTextColor = 'white';
muiTheme.snackbar.backgroundColor = 'rgba(255, 30, 30, 0.9)';
muiTheme.snackbar.textColor = 'rgba(255, 255, 255, 0.75)';
muiTheme.stepper.textColor = '#eee';
muiTheme.stepper.disabledTextColor = '#777';
muiTheme.tabs = lightTheme.tabs;
muiTheme.tabs.backgroundColor = 'transparent';
muiTheme.tabs.selectedTextColor = 'white';
muiTheme.tabs.textColor = 'rgba(255, 255, 255, 0.5)';
muiTheme.textField.floatingLabelColor = 'rgba(255, 255, 255, 0.5)';
muiTheme.textField.hintColor = 'rgba(255, 255, 255, 0.5)';
muiTheme.textField.disabledTextColor = muiTheme.textField.textColor;
muiTheme.toolbar = lightTheme.toolbar;
muiTheme.toolbar.backgroundColor = 'transparent';
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
