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

import { nextTooltip } from './actions';

import styles from './tooltips.css';

class Tooltips extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    currentId: PropTypes.number,
    onNextTooltip: PropTypes.func
  }

  componentDidMount () {
    const { onNextTooltip } = this.props;

    onNextTooltip();
    this.redirect();
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.currentId !== this.props.currentId) {
      this.redirect(nextProps);
    }
  }

  redirect (props = this.props) {
    const { currentId } = props;

    if (currentId !== undefined && currentId !== -1) {
      const viewLink = '/accounts/';

      this.context.router.push(viewLink);
    }
  }

  render () {
    const { currentId } = this.props;

    if (currentId === -1) {
      return null;
    }

    return (
      <div className={ styles.overlay } />
    );
  }
}

function mapStateToProps (state) {
  const { currentId } = state.tooltip;

  return {
    currentId
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onNextTooltip: nextTooltip
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Tooltips);
