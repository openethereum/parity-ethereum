import React, { Component, PropTypes } from 'react';

import { Toolbar, ToolbarGroup, ToolbarTitle } from 'material-ui/Toolbar';

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    blockNumber: PropTypes.string,
    totalSupply: PropTypes.string,
    remaining: PropTypes.string,
    price: PropTypes.string
  }

  render () {
    if (!this.props.totalSupply) {
      return null;
    }

    return (
      <Toolbar className='toolbar'>
        <ToolbarGroup>
          <ToolbarTitle text='GAVcoin' />
        </ToolbarGroup>
        <ToolbarGroup>
          <p>
            #{ this.props.blockNumber }: { this.props.remaining } / { this.props.totalSupply } available
          </p>
        </ToolbarGroup>
      </Toolbar>
    );
  }
}
