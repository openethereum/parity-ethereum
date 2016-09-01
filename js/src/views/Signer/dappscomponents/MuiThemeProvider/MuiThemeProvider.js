
import React, { Component, PropTypes } from 'react';

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
        { this.props.children }
      </MuiThemeProvider>
    );
  }

  static propTypes = {
    children: PropTypes.object.isRequired
  }

}
