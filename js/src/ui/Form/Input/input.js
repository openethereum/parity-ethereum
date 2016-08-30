import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';

// TODO: duplicated in Select
const UNDERLINE_DISABLED = {
  borderColor: 'rgba(255, 255, 255, 0.298039)' // 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderBottom: 'solid 2px'
};

const NAME_ID = ' ';

export default class Input extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    multiLine: PropTypes.bool,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onKeyDown: PropTypes.func,
    rows: PropTypes.number,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ])
  }

  render () {
    return (
      <TextField
        autoComplete='off'
        className={ this.props.className }
        disabled={ this.props.disabled }
        errorText={ this.props.error }
        floatingLabelFixed
        floatingLabelText={ this.props.label }
        fullWidth
        hintText={ this.props.hint }
        multiLine={ this.props.multiLine }
        name={ NAME_ID }
        id={ NAME_ID }
        rows={ this.props.rows }
        type={ this.props.type || 'text' }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ this.props.value }
        onBlur={ this.props.onBlur }
        onChange={ this.props.onChange }
        onKeyDown={ this.props.onKeyDown }>
        { this.props.children }
      </TextField>
    );
  }
}
