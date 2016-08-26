import React, { Component, PropTypes } from 'react';

import { formatBlockNumber } from '../../format';

export default class ColumnBlockNumber extends Component {
  static propTypes = {
    blockNumber: PropTypes.object
  }

  render () {
    const { blockNumber } = this.props;

    return (
      <td className='blocknumber'>
        { formatBlockNumber(blockNumber) }
      </td>
    );
  }
}
