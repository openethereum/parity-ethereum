import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';
const muiTheme = getMuiTheme(lightBaseTheme);

import CircularProgress from 'material-ui/CircularProgress';
import Status from '../Status';
import Lookup from '../Lookup';

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object
  };
  getChildContext () {
    return { muiTheme };
  }

  render () {
    const { contract, fee, owner, actions } = this.props;

    if (!contract || !fee || !owner) {
      return (<CircularProgress size={ 1 } />);
    }
    return (
      <div>
        <Status address={ contract.address } fee={ fee } owner={ owner } />
        <Lookup lookup={ this.props.lookup } actions={ actions.lookup } />
      </div>
    );
  }

}

Application.propTypes = {
  contract: PropTypes.object,
  fee: PropTypes.object,
  owner: PropTypes.string
};
