import React, { Component, PropTypes } from 'react';

import { Toolbar, ToolbarGroup, ToolbarTitle } from 'material-ui/Toolbar';

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    blockNumber: PropTypes.object,
    totalSupply: PropTypes.object,
    remaining: PropTypes.object,
    price: PropTypes.object
  }

  render () {
    if (!this.props.totalSupply) {
      return null;
    }

    return (
      <Toolbar className='status'>
        <ToolbarGroup>
          <ToolbarTitle text='GAVcoin' />
        </ToolbarGroup>
        <ToolbarGroup>
          <p>
            #{ this.props.blockNumber.toFormat() }: { this.props.remaining.toFormat() } / { this.props.totalSupply.toFormat() } available
          </p>
        </ToolbarGroup>
      </Toolbar>
    );
  }
}
