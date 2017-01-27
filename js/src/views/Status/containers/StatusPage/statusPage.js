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

import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { clearStatusLogs, toggleStatusLogs, toggleStatusRefresh } from '~/redux/actions';

import Debug from '../../components/Debug';
import Status from '../../components/Status';

import styles from './statusPage.css';

class StatusPage extends Component {
  static propTypes = {
    nodeStatus: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired
  }

  componentWillMount () {
    this.props.actions.toggleStatusRefresh(true);
  }

  componentWillUnmount () {
    this.props.actions.toggleStatusRefresh(false);
  }

  render () {
    return (
      <div className={ styles.body }>
        <Status { ...this.props } />
        <Debug { ...this.props } />
      </div>
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators({
      clearStatusLogs,
      toggleStatusLogs,
      toggleStatusRefresh
    }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(StatusPage);
