import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';

const UNDERLINE_DISABLED = {
  borderColor: 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderWidth: '2px'
};

export default class Input extends Component {
  static propTypes = {
    disabled: PropTypes.bool,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string,
    multiLine: PropTypes.bool,
    onChange: PropTypes.func,
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
        disabled={ this.props.disabled }
        floatingLabelText={ this.props.floatingLabelText }
        fullWidth
        hintText={ this.props.hintText }
        multiLine={ this.props.multiLine }
        rows={ this.props.rows }
        type={ this.props.type || 'text' }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ this.props.value }
        onChange={ this.props.onChange } />
    );
  }
}
