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
import { isReactComponent } from '../../util/react';
import ReactCSSTransitionGroup from 'react-addons-css-transition-group';
import './AnimateChildren.css';

export default class AnimateChildren extends Component {
  render () {
    const className = this.props.absolute ? 'absoluteAnimationContainer' : '';
    return (
      <ReactCSSTransitionGroup
        component='div'
        className={ className }
        transitionName='transition'
        transitionAppear
        transitionAppearTimeout={ 0 }
        transitionLeaveTimeout={ 0 }
        transitionEnterTimeout={ 0 }
        >
        { this.renderChildren() }
      </ReactCSSTransitionGroup>
    );
  }

  renderChildren () {
    const { children, isView } = this.props;

    if (isView) {
      return React.cloneElement(this.props.children, {
        key: this.props.pathname
      });
    }

    if (isReactComponent(children)) {
      return React.cloneElement(this.props.children, { ...this.props });
    }

    return children;
  }

  static propTypes = {
    children: PropTypes.any.isRequired,
    pathname: PropTypes.string,
    isView: PropTypes.bool,
    absolute: PropTypes.bool
  }

}
