import React, { Component, PropTypes } from 'react';

import { Toolbar, ToolbarGroup, ToolbarTitle } from 'material-ui/Toolbar';

import { formatCoins, formatEth } from '../format';

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    gavBalance: PropTypes.object,
    blockNumber: PropTypes.object,
    totalSupply: PropTypes.object,
    remaining: PropTypes.object,
    price: PropTypes.object
  }

  render () {
    if (!this.props.totalSupply) {
      return null;
    }

    const { blockNumber, totalSupply, remaining, price } = this.props;

    return (
      <Toolbar className='status'>
        <ToolbarGroup>
          <ToolbarTitle text='GAVcoin' />
        </ToolbarGroup>
        <ToolbarGroup>
          <p>
            #{ blockNumber.toFormat() }: { formatCoins(remaining, 0) } available @ { formatEth(price) }ΞTH, { formatCoins(totalSupply, 0) } minted
          </p>
        </ToolbarGroup>
      </Toolbar>
    );
  }
}
