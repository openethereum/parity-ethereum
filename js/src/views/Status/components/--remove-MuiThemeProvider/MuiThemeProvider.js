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

import React, { Component, PropTypes } from 'react';
import injectTapEventPlugin from 'react-tap-event-plugin';
// Needed for onTouchTap, for material ui
// http://stackoverflow.com/a/34015469/988941
injectTapEventPlugin();

import { deepOrange500 } from 'material-ui/styles/colors';
import getMuiTheme from 'material-ui/styles/getMuiTheme';
import MuiThemeProvider from 'material-ui/styles/MuiThemeProvider';

const muiTheme = getMuiTheme({
  fontFamily: '"Source Sans Pro", "Helvetica Neue", arial, sans-serif',
  palette: {
    primary1Color: '#6691C2',
    accent1Color: deepOrange500
  }
});

export default class WrappedMuiThemeProvider extends Component {

  render () {
    return (
      <MuiThemeProvider muiTheme={ muiTheme }>
        { this.props.children && React.cloneElement(this.props.children, {
          ...this.props
        }) }
      </MuiThemeProvider>
    );
  }

  static propTypes = {
    children: PropTypes.object.isRequired
  }

}
