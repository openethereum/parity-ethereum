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
    onSubmit: PropTypes.func,
    rows: PropTypes.number,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ])
  }

  state = {
    value: this.props.value
  }

  render () {
    const { value } = this.state;
    const { children, className, disabled, error, label, hint, multiLine, rows, type } = this.props;

    return (
      <TextField
        autoComplete='off'
        className={ className }
        disabled={ disabled }
        errorText={ error }
        floatingLabelFixed
        floatingLabelText={ label }
        fullWidth
        hintText={ hint }
        multiLine={ multiLine }
        name={ NAME_ID }
        id={ NAME_ID }
        rows={ rows }
        type={ type || 'text' }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ value }
        onBlur={ this.onBlur }
        onChange={ this.onChange }
        onKeyDown={ this.onKeyDown }>
        { children }
      </TextField>
    );
  }

  onChange = (event, value) => {
    this.setValue(value);

    this.props.onChange && this.props.onChange(event, value);
  }

  onBlur = (event) => {
    const { value } = event.target;

    this.onSubmit(value);

    this.props.onBlur && this.props.onBlur(event);
  }

  onKeyDown = (event) => {
    const { value } = event.target;

    if (event.which === 13) {
      this.onSubmit(value);
    }

    this.props.onKeyDown && this.props.onKeyDown(event);
  }

  onSubmit = (value) => {
    console.log('onSubmit', value, this.props.onSubmit);
    this.setValue(value);

    this.props.onSubmit && this.props.onSubmit(value);
  }

  setValue (value) {
    this.setState({
      value
    });
  }
}
