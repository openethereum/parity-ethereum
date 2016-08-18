import React, { Component, PropTypes } from 'react';
import { SelectField } from 'material-ui';

// TODO: duplicated in Input
const UNDERLINE_DISABLED = {
  borderColor: 'rgba(255, 255, 255, 0.298039)' // 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderBottom: 'solid 2px'
};

const NAME_ID = ' ';

export default class Select extends Component {
  static propTypes = {
    children: PropTypes.node,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onKeyDown: PropTypes.func,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ])
  }

  render () {
    return (
      <SelectField
        autoComplete='off'
        disabled={ this.props.disabled }
        errorText={ this.props.error }
        floatingLabelFixed
        floatingLabelText={ this.props.label }
        fullWidth
        hintText={ this.props.hint }
        name={ NAME_ID }
        id={ NAME_ID }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ this.props.value }
        onBlur={ this.props.onBlur }
        onChange={ this.props.onChange }
        onKeyDown={ this.props.onKeyDown }>
        { this.props.children }
      </SelectField>
    );
  }
}
