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

import GeoPattern from 'geopattern';
import getMuiTheme from 'material-ui/styles/getMuiTheme';

import Loading from '../Loading';
import Status from '../Status';
import Tokens from '../Tokens';
import Actions from '../Actions';

import styles from './application.css';

const muiTheme = getMuiTheme({
  palette: {
    primary1Color: '#27ae60'
  }
});

export default class Application extends Component {
  static childContextTypes = {
    muiTheme: PropTypes.object
  }

  static propTypes = {
    isLoading: PropTypes.bool,
    contract: PropTypes.object
  };

  render () {
    const { isLoading, contract } = this.props;

    if (isLoading) {
      return (
        <Loading />
      );
    }

    return (
      <div className={ styles.application } style={ this.getBackgroundStyle() }>
        <Status
          address={ contract.address }
          fee={ contract.fee } />

        <Actions />

        <Tokens />
      </div>
    );
  }

  getBackgroundStyle () {
    let seed = this.props.contract ? this.props.contract.address : '0x0';
    const url = GeoPattern.generate(seed).toDataUrl();

    return {
      background: `linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5)), ${url}`
    };
  }

  getChildContext () {
    return {
      muiTheme
    };
  }

}
