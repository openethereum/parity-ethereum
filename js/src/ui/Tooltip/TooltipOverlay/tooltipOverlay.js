import React, { Component, PropTypes } from 'react';

import styles from '../style.css';

export default class TooltipOverlay extends Component {
  static childContextTypes = {
    tooltips: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    currentId: 0,
    updateCallbacks: []
  }

  render () {
    const overlay = this.state.currentId === -1 ? null : (<div className={ styles.overlay } />);

    return (
      <div>
        { overlay }
        { this.props.children }
      </div>
    );
  }

  getChildContext () {
    return {
      tooltips: this
    };
  }

  register (updateCallback) {
    if (this.state.currentId === -1) {
      return;
    }

    this.state.updateCallbacks.push(updateCallback);
    this.update();

    return this.state.updateCallbacks.length - 1;
  }

  update = () => {
    this.state.updateCallbacks.forEach((cb) => {
      cb(this.state.currentId, this.state.updateCallbacks.length - 1);
    });
  }

  next () {
    this.setState({
      currentId: this.state.currentId + 1
    }, this.update);
  }

  close () {
    this.setState({
      currentId: -1
    }, this.update);
  }
}
