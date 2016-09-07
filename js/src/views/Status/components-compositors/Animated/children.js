
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
