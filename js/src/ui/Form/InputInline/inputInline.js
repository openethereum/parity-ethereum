import React, { Component, PropTypes } from 'react';

import Input from '../Input';

import styles from '../style.css';

export default class InputInline extends Component {
  static propTypes = {
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
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
    if (!this.state.editing) {
      return (
        <div
          className={ styles.inlineedit }
          onClick={ this.onEdit }>
          { this.props.static || this.props.value }
        </div>
      );
    }

    return (
      <Input
        error={ this.props.error }
        label={ this.props.label }
        hint={ this.props.hint }
        type={ this.props.type }
        value={ this.props.value }
        onBlur={ this.onEdit }
        onChange={ this.props.onChange }
        onKeyDown={ this.onKeyDown } />
    );
  }

  onEdit = () => {
    this.setState({
      editing: !this.state.editing
    });
  }

  onKeyDown = (event) => {
    if (event.keyCode === 13) {
      this.onEdit();
    }
  }
}
