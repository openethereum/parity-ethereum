import React, { Component } from 'react';

import RpcAutoComplete from '../../../RpcAutoComplete';

export default class RpcAutoCompletePage extends Component {

  state = {
    selected: null
  }

  render () {
    return (
      <div>
        <h1>RpcAutoComplete</h1>
        <RpcAutoComplete onNewRequest={ this.onNewRequest } />
        { this.renderSelected() }
      </div>
    );
  }

  renderSelected () {
    const { selected } = this.state;
    if (!selected) {
      return (
        <h3>Select a method above</h3>
      );
    }

    return (
      <h3>You have selected:
        <strong> { selected }</strong>
      </h3>
    );
  }

  onNewRequest = selected => {
    this.setState({ selected });
  }

}
