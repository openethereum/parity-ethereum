import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';

const UNDERLINE_DISABLED = {
  borderColor: 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderBottom: 'solid 2px'
};

export default class Input extends Component {
  static propTypes = {
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    multiLine: PropTypes.bool,
    onChange: PropTypes.func,
    rows: PropTypes.number,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ])
  }

  render () {
    const nameid = ' ';

    return (
      <div>
        <TextField
          autoComplete='off'
          disabled={ this.props.disabled }
          errorText={ this.props.error }
          floatingLabelFixed
          floatingLabelText={ this.props.label }
          fullWidth
          hintText={ this.props.hint }
          multiLine={ this.props.multiLine }
          name={ nameid }
          id={ nameid }
          rows={ this.props.rows }
          type={ this.props.type || 'text' }
          underlineDisabledStyle={ UNDERLINE_DISABLED }
          underlineStyle={ UNDERLINE_NORMAL }
          value={ this.props.value }
          onChange={ this.props.onChange } />
      </div>
    );
  }
}
