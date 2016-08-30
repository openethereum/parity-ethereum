import React, { Component, PropTypes } from 'react';

import Input from '../Input';

import styles from '../style.css';

export default class InputInline extends Component {
  static propTypes = {
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ]),
    static: PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ])
  }

  state = {
    editing: false
  }

  render () {
    const { editing } = this.state;
    const { error, label, hint, type, value } = this.props;

    if (!editing) {
      return (
        <div
          className={ styles.inlineedit }
          onClick={ this.onToggle }>
          { this.props.static || value }
        </div>
      );
    }

    return (
      <Input
        error={ error }
        label={ label }
        hint={ hint }
        type={ type }
        value={ value }
        onBlur={ this.onBlur }
        onChange={ this.props.onChange }
        onKeyDown={ this.onKeyDown } />
    );
  }

  onBlur = () => {
    this.onToggle();

    if (this.props.onBlur) {
      this.props.onBlur();
    }
  }

  onToggle = () => {
    this.setState({
      editing: !this.state.editing
    });
  }

  onKeyDown = (event) => {
    if (event.keyCode === 13) {
      this.onToggle();
    }
  }
}
