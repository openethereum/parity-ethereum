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
// import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router-dom';

/** Additional Libraries **/
import { Dropdown as SemanticDropdown } from 'semantic-ui-react';
import store from 'store';

/** Assets **/
import Currency from '../Assets/currency.png';
import Proxy from '../Assets/stealth.png';
import Node from '../Assets/node.png';

/** Stylesheets **/
import styles from './home.css';

export default class Home extends Component {
  static propTypes = {
    history: PropTypes.object.isRequired,
    location: PropTypes.object.isRequired,
    match: PropTypes.object.isRequired
  }

  render () {
    return (
      <div className={ styles.home }>
        <div className={ styles.homeSpacer } />

        <div className={ styles.homeCard }>
          <div onClick={ this.homeClick } className={ styles.homeContainer }>
            <div className={ styles.settingsText }>Settings</div>

            <div className={ styles.settingsTablature }>

              <div className={ styles.table }>
                <div className={ styles.tableLeft }>
                  <div className={ styles.tableIcon }>
                    <img src={ Currency } alt='currency' />
                  </div>
                  <div className={ styles.tableName }>
                    Currency
                  </div>
                </div>
                <div className={ styles.tableDetail }>
                  <CurrencyDropdown />
                </div>
              </div>

              <Link to='/proxy'>
                <div className={ styles.clickableTable }>
                  <div className={ styles.tableLeft }>
                    <div className={ styles.tableIcon }>
                      <img src={ Proxy } alt='proxy' />
                    </div>
                    <div className={ styles.tableName }>
                      Proxy
                    </div>
                  </div>
                  <div className={ styles.tableDetail }>
                    Setup an alternate address
                  </div>
                  <div className={ styles.tableArrow } />
                </div>
              </Link>

              <Link to='/parity'>
                <div className={ styles.clickableTable }>
                  <div className={ styles.tableLeft }>
                    <div className={ styles.tableIcon }>
                      <img src={ Node } alt='node' />
                    </div>
                    <div className={ styles.tableName }>
                      Parity Node
                    </div>
                  </div>
                  <div className={ styles.tableDetail }>
                    Edit your node config
                  </div>
                  <div className={ styles.tableArrow } />
                </div>
              </Link>

            </div>
          </div>
        </div>

        <div className={ styles.homeSpacer } />
      </div>
    );
  }
}

class CurrencyDropdown extends Component {
  constructor () {
    super();
    // Get state from store, if none, use USD
    let currency = store.get('parity::currency');

    if (!currency || currency.value === '' || typeof currency.value === 'object') {
      currency = 'USD';
    } else {
      currency = currency.value;
    }

    this.state = {
      value: currency
    };
  }

  currencyChange = (e, dropdown) => {
    // update state
    this.setState({ value: dropdown.value });
    // update store
    store.set('parity::currency', { value: dropdown.value });
  }

  render () {
    return (
      <div className={ styles.currency }>
        <SemanticDropdown
          className={ styles.currencySemantic }
          placeholder={ this.state.value }
          selection
          options={ [
            { key: 'USD', value: 'USD', text: '$ USD' },
            { key: 'GBP', value: 'GBP', text: '£ GBP' },
            { key: 'EUR', value: 'EUR', text: '€ EUR' }
          ] }
          onChange={ this.currencyChange }
        />
      </div>
    );
  }
}
