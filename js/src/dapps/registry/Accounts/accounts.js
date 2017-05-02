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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { AccountsIcon } from '~/ui/Icons';

import { init } from './actions';
import IdentityIcon from '../IdentityIcon';

import styles from './accounts.css';

class Accounts extends Component {
  static propTypes = {
    selected: PropTypes.oneOfType([
      PropTypes.oneOf([ null ]),
      PropTypes.string
    ]),
    onInit: PropTypes.func.isRequired
  };

  componentWillMount () {
    this.props.onInit();
  }

  render () {
    const { selected } = this.props;

    if (!selected) {
      return (
        <AccountsIcon
          className={ styles.icon }
          color='white'
        />
      );
    }

    return (
      <IdentityIcon
        className={ styles.icon }
        address={ selected }
      />
    );
  }
}

const mapStateToProps = (state) => state.accounts;
const mapDispatchToProps = (dispatch) => bindActionCreators({
  onInit: init
}, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(Accounts);
