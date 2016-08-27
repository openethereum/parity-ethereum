import React, { Component, PropTypes } from 'react';

import styles from '../style.css';

const LS_KEY = 'tooltips';

export default class TooltipOverlay extends Component {
  static childContextTypes = {
    tooltips: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    currentId: -1,
    updateCallbacks: []
  }

  componentDidMount () {
    const ls = window.localStorage.getItem('tooltips');

    this.setState({
      currentId: ls ? -1 : 0
    });
  }

  render () {
    const overlay = this.state.currentId === -1 ? null : (<div className={ styles.overlay } />);

    return (
      <div className={ styles.container }>
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
    window.localStorage.setItem(LS_KEY, '{"state":"off"}');

    this.setState({
      currentId: -1
    }, this.update);
  }
}
