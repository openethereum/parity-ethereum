import React, { Component } from 'react';
import { connect } from 'react-redux';

import InputText from './input-text';

class InputTextContainer extends Component {

  render () {
    return (<InputText
      { ...this.props }
    />);
  }
}

const mapStateToProps = (state) => {
  const { contract } = state.status;

  return { contract };
};

export default connect(mapStateToProps)(InputTextContainer);
