import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';
const muiTheme = getMuiTheme(lightBaseTheme);

import Loading from '../Loading';
import Status from '../Status';

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object
  };
  getChildContext () {
    return { muiTheme };
  }

  render () {
    const { contract, fee, owner } = this.props;

    if (!contract || !fee || !owner) {
      return (<Loading />);
    }
    return (
      <Status address={ contract.address } fee={ fee } owner={ owner } />
    );
  }

}

Application.propTypes = {
  contract: PropTypes.object,
  fee: PropTypes.object,
  owner: PropTypes.string
};
