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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { Checkbox, Dropdown, Form, Input, Warning } from '@parity/ui';

import Price from '../Price';
import { WARNING_NO_PRICE } from '../store';
import styles from './optionsStep.css';

const WARNING_LABELS = {
  [WARNING_NO_PRICE]: (
    <FormattedMessage
      id='shapeshift.warning.noPrice'
      defaultMessage='No price match was found for the selected type'
    />
  )
};

@observer
export default class OptionsStep extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  };

  render () {
    const { coinSymbol, hasAcceptedTerms, price, refundAddress, warning } = this.props.store;
    let { coins } = this.props.store;

    if (!coins.length) {
      return (
        <div className={ styles.empty }>
          <FormattedMessage
            id='shapeshift.optionsStep.noPairs'
            defaultMessage='There are currently no exchange pairs/coins available to fund with.'
          />
        </div>
      );
    }

    coins = coins.map(this.renderCoinSelectItem);

    return (
      <div className={ styles.body }>
        <Form>
          <Dropdown
            className={ styles.coinselector }
            hint={
              <FormattedMessage
                id='shapeshift.optionsStep.typeSelect.hint'
                defaultMessage='the type of crypto conversion to do'
              />
            }
            label={
              <FormattedMessage
                id='shapeshift.optionsStep.typeSelect.label'
                defaultMessage='fund account from'
              />
            }
            options={ coins }
            onChange={ this.onSelectCoin }
            value={ coinSymbol }
          />
          <Input
            hint={
              <FormattedMessage
                id='shapeshift.optionsStep.returnAddr.hint'
                defaultMessage='the return address for send failures'
              />
            }
            label={
              <FormattedMessage
                id='shapeshift.optionsStep.returnAddr.label'
                defaultMessage='(optional) {coinSymbol} return address'
                values={ { coinSymbol } }
              />
            }
            onSubmit={ this.onChangeRefundAddress }
            value={ refundAddress }
          />
          <Checkbox
            checked={ hasAcceptedTerms }
            className={ styles.accept }
            label={
              <FormattedMessage
                id='shapeshift.optionsStep.terms.label'
                defaultMessage='I understand that ShapeShift.io is a 3rd-party service and by using the service any transfer of information and/or funds is completely out of the control of Parity'
              />
            }
            onClick={ this.onToggleAcceptTerms }
          />
        </Form>
        <Warning warning={ WARNING_LABELS[warning] } />
        <Price
          coinSymbol={ coinSymbol }
          price={ price }
        />
      </div>
    );
  }

  renderCoinSelectItem = (coin) => {
    const { image, name, symbol } = coin;

    return {
      image,
      text: name,
      value: symbol
    };
  }

  onChangeRefundAddress = (event, refundAddress) => {
    this.props.store.setRefundAddress(refundAddress);
  }

  onSelectCoin = (event, value) => {
    this.props.store.setCoinSymbol(value);
  }

  onToggleAcceptTerms = () => {
    this.props.store.toggleAcceptTerms();
  }
}
