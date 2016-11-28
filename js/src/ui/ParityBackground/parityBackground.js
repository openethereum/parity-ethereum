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
import { connect } from 'react-redux';

class ParityBackground extends Component {
  static propTypes = {
    style: PropTypes.object.isRequired,
    children: PropTypes.node,
    className: PropTypes.string,
    onClick: PropTypes.func
  };

  render () {
    const { children, className, style, onClick } = this.props;

    return (
      <div
        className={ className }
        style={ style }
        onTouchTap={ onClick }>
        { children }
      </div>
    );
  }
}

function mapStateToProps (_, initProps) {
  const { gradient, seed, muiTheme } = initProps;

  let _seed = seed;
  let _props = { style: muiTheme.parity.getBackgroundStyle(gradient, seed) };

  return (state, props) => {
    const { backgroundSeed } = state.settings;
    const { seed } = props;

    const newSeed = seed || backgroundSeed;

    if (newSeed === _seed) {
      return _props;
    }

    _seed = newSeed;
    _props = { style: muiTheme.parity.getBackgroundStyle(gradient, newSeed) };

    return _props;
  };
}

export default connect(
  mapStateToProps
)(ParityBackground);
