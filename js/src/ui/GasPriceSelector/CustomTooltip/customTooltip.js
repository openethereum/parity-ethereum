// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

export default class CustomTooltip extends Component {
  static propTypes = {
    histogram: PropTypes.object.isRequired,
    intl: PropTypes.object.isRequired,
    active: PropTypes.bool,
    label: PropTypes.number,
    payload: PropTypes.array,
    type: PropTypes.string
  };

  static childContextTypes = {
    intl: PropTypes.object.isRequired
  };

  render () {
    const { active, label, histogram } = this.props;

    if (!active) {
      return null;
    }

    const index = label;

    const count = histogram.counts[index];
    const minprice = histogram.bucketBounds[index];
    const maxprice = histogram.bucketBounds[index + 1];

    return (
      <div>
        <p className='label'>
          <FormattedMessage
            id='ui.gasPriceSelector.customTooltip.transactions'
            defaultMessage='{number} {number, plural, one {transaction} other {transactions}} with gas price set from {minPrice} to {maxPrice}'
            values={ {
              number: count.toNumber(),
              minPrice: <span>{ minprice.toFormat(0) }</span>,
              maxPrice: <span>{ maxprice.toFormat(0) }</span>
            } }
          />
        </p>
      </div>
    );
  }

  getChildContext () {
    return { intl: this.props.intl };
  }
}
