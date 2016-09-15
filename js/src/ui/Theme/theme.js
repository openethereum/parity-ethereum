// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import darkBaseTheme from 'material-ui/styles/baseThemes/darkBaseTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const lightTheme = getMuiTheme(lightBaseTheme);
const muiTheme = getMuiTheme(darkBaseTheme);

muiTheme.stepper.textColor = '#eee';
muiTheme.stepper.disabledTextColor = '#777';
muiTheme.inkBar.backgroundColor = 'transparent'; // 'rgb(0, 151, 167)'; // 'rgba(255, 136, 0, 0.8)';
muiTheme.raisedButton.primaryTextColor = 'white';
muiTheme.snackbar.backgroundColor = 'rgba(255, 30, 30, 0.9)';
muiTheme.snackbar.textColor = 'rgba(255, 255, 255, 0.9)';
muiTheme.tabs = lightTheme.tabs;
muiTheme.tabs.backgroundColor = 'rgb(65, 65, 65)';
muiTheme.tabs.selectedTextColor = 'rgb(255, 255, 255)'; // 'rgb(0, 151, 167)'; // 'rgba(255, 136, 0, 0.8)';
muiTheme.tabs.textColor = 'rgb(0, 151, 167)'; // 'rgba(255, 255, 255, 1)'; // 'rgba(0, 151, 167, 1)';
muiTheme.textField.disabledTextColor = muiTheme.textField.textColor;
muiTheme.toolbar = lightTheme.toolbar;
muiTheme.toolbar.backgroundColor = 'rgb(80, 80, 80)'; // 'rgba(255, 136, 0, 0.5)'; // 'rgb(80, 80, 80)';

export default muiTheme;
