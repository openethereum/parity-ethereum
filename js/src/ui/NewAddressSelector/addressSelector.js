// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import ReactDOM from 'react-dom';
import { connect } from 'react-redux';
import Portal from 'react-portal';
import keycode, { codes } from 'keycode';

import CloseIcon from 'material-ui/svg-icons/navigation/close';

import IdentityIcon from '~/ui/IdentityIcon';
import InputAddress from '~/ui/Form/InputAddress';
import { fromWei } from '~/api/util/wei';

import styles from './addressSelector.css';

class AddressSelector extends Component {
  static propTypes = {
    // Required props
    onChange: PropTypes.func.isRequired,

    // Redux props
    accountsInfo: PropTypes.object,
    accounts: PropTypes.object,
    balances: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    wallets: PropTypes.object,

    // Optional props
    allowInput: PropTypes.bool,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    value: PropTypes.string
  };

  static defaultProps = {
    value: ''
  };

  state = {
    inputValue: '',
    values: [],
    expanded: false,
    top: 0,
    left: 0,
    focusedCat: null,
    focusedItem: null
  };

  componentWillMount () {
    this.setValues();
  }

  componentWillReceiveProps (nextProps) {
    if (this.values && this.values.length > 0) {
      return;
    }

    this.setValues(nextProps);
  }

  setValues (props = this.props) {
    const { accounts = {}, contracts = {}, contacts = {} } = props;

    if (Object.keys(accounts).length + Object.keys(contracts).length + Object.keys(contacts).length === 0) {
      return;
    }

    this.values = [
      {
        label: 'accounts',
        values: Object.values(accounts)
      },
      {
        label: 'contacts',
        values: Object.values(contacts)
      },
      {
        label: 'contracts',
        values: Object.values(contracts)
      }
    ];

    this.handleChange();
  }

  render () {
    const input = this.renderInput();
    const content = this.renderContent();

    return (
      <div className={ styles.main } ref='main'>
        { input }
        { content }
      </div>
    );
  }

  renderInput () {
    const { accountsInfo, disabled, error, hint, label, value } = this.props;

    return (
      <div
        className={ styles.inputAddress }
        onClick={ this.handleFocus }
        onFocus={ this.handleFocus }
        onKeyDown={ this.handleInputAddresKeydown }
        ref='inputAddress'
        tabIndex={ 0 }
      >
        <InputAddress
          accountsInfo={ accountsInfo }
          error={ error }
          label={ label }
          hint={ hint }
          tabIndex={ -1 }
          value={ value }

          allowCopy={ false }
          disabled
          text
        />
      </div>
    );
  }

  renderContent () {
    const { hint } = this.props;
    const { expanded, top, left } = this.state;

    const classes = [ styles.overlay ];

    if (expanded) {
      classes.push(styles.expanded);
    }

    return (
      <Portal isOpened onClose={ this.handleClose }>
        <div
          className={ classes.join(' ') }
          style={ { top, left } }
          onKeyDown={ this.handleKeyDown }
        >
          <div className={ styles.inputContainer }>
            <input
              className={ styles.input }
              placeholder={ hint }

              onFocus={ this.handleInputFocus }
              onChange={ this.handleChange }

              ref={ this.setInputRef }
            />

            { this.renderCurrentInput() }
            { this.renderAccounts() }
            { this.renderCloseIcon() }
          </div>
        </div>
      </Portal>
    );
  }

  renderCurrentInput () {
    if (!this.props.allowInput) {
      return null;
    }

    const { inputValue } = this.state;

    if (inputValue.length === 0 || !/^(0x)?[a-f0-9]*$/i.test(inputValue)) {
      return null;
    }

    return (
      <div

      >
        { this.renderAccountCard({ address: inputValue }) }
      </div>
    );
  }

  renderCloseIcon () {
    const { expanded } = this.state;

    if (!expanded) {
      return null;
    }

    return (
      <div className={ styles.closeIcon } onClick={ this.handleClose }>
        <CloseIcon style={ { width: 48, height: 48 } } />
      </div>
    );
  }

  renderAccounts () {
    const { expanded, values } = this.state;

    if (!expanded) {
      // return null;
    }

    if (values.length === 0) {
      return (
        <div className={ styles.categories }>
          <div className={ styles.empty }>
            No account matches this query...
          </div>
        </div>
      );
    }

    const categories = values.map((category) => {
      return this.renderCategory(category.label, category.values);
    });

    return (
      <div className={ styles.categories }>
        { categories }
      </div>
    );
  }

  renderCategory (name, values = []) {
    if (values.length === 0) {
      return null;
    }

    const cards = values
      .map((account) => this.renderAccountCard(account));

    return (
      <div className={ styles.category } key={ name }>
        <div className={ styles.title }>{ name }</div>
        <div className={ styles.cards }>
          <div>{ cards }</div>
        </div>
      </div>
    );
  }

  renderAccountCard (_account) {
    const { address, index = null } = _account;

    const account = this.props.accountsInfo[address];
    const name = (account && account.name && account.name.toUpperCase()) || address;
    const balance = this.renderBalance(address);

    const onClick = () => {
      this.handleClick(address);
    };

    const onFocus = () => {
      this.setState({ focusedItem: index });
    };

    const classes = [ styles.account ];

    const addressElements = name !== address
      ? (
        <div className={ styles.address }>{ address }</div>
      )
      : null;

    return (
      <div
        key={ address }
        ref={ `account_${index}` }
        tabIndex={ 0 }
        className={ classes.join(' ') }
        onClick={ onClick }
        onFocus={ onFocus }
      >
        <IdentityIcon address={ address } />
        <div className={ styles.accountInfo }>
          <div className={ styles.accountName }>{ name }</div>
          { addressElements }
          { balance }
        </div>
      </div>
    );
  }

  renderBalance (address) {
    const { balances = {} } = this.props;

    const balance = balances[address];

    if (!balance || !balance.tokens) {
      return null;
    }

    const ethToken = balance.tokens
      .find((tok) => tok.token && (tok.token.tag || '').toLowerCase() === 'eth');

    if (!ethToken) {
      return null;
    }

    const value = fromWei(ethToken.value).toFormat(3);

    return (
      <div className={ styles.balance }>
        <span className={ styles.value }>{ value }</span>
        <span className={ styles.tag }>ETH</span>
      </div>
    );
  }

  setInputRef = (refId) => {
    this.inputRef = refId;
  }

  handleCustomInput = () => {
    const { allowInput } = this.props;
    const { inputValue, values } = this.state;

    // If input is HEX and allowInput === true, send it
    if (allowInput && inputValue && /^(0x)?([0-9a-f])+$/i.test(inputValue)) {
      return this.handleClick(inputValue);
    }

    // If only one value, select it
    if (values.length === 1 && values[0].values.length === 1) {
      const value = values[0].values[0];
      return this.handleClick(value.address);
    }
  }

  handleInputAddresKeydown = (event) => {
    const code = keycode(event);

    // Simulate click on input address if enter is pressed
    if (code === 'enter') {
      return this.handleDOMAction('inputAddress', 'click');
    }
  }

  handleKeyDown = (event) => {
    const codeName = keycode(event);

    switch (codeName) {
      case 'esc':
        event.preventDefault();
        return this.handleClose();

      case 'enter':
        const index = this.state.focusedItem;
        if (!index) {
          return this.handleCustomInput();
        }

        return this.handleDOMAction(`account_${index}`, 'click');

      case 'left':
      case 'right':
      case 'up':
      case 'down':
        event.preventDefault();
        return this.handleNavigation(codeName);

      default:
        const code = codes[codeName];

        // @see https://github.com/timoxley/keycode/blob/master/index.js
        // lower case chars
        if (code >= (97 - 32) && code <= (122 - 32)) {
          return this.handleDOMAction(this.inputRef, 'focus');
        }

        // numbers
        if (code >= 48 && code <= 57) {
          return this.handleDOMAction(this.inputRef, 'focus');
        }

        return event;
    }
  }

  handleDOMAction = (ref, method) => {
    const refItem = typeof ref === 'string' ? this.refs[ref] : ref;
    const element = ReactDOM.findDOMNode(refItem);

    if (!element || typeof element[method] !== 'function') {
      console.warn('could not find', ref, 'or method', method);
      return;
    }

    return element[method]();
  }

  focusItem = (index) => {
    this.setState({ focusedItem: index });
    return this.handleDOMAction(`account_${index}`, 'focus');
  }

  handleNavigation = (direction) => {
    const { focusedItem, focusedCat, values } = this.state;

    // Don't do anything if no values
    if (values.length === 0) {
      return;
    }

    // Focus on the first element if none selected yet if going down
    if (!focusedItem) {
      if (direction !== 'down') {
        return;
      }

      const nextValues = values[focusedCat || 0];
      const nextFocus = nextValues ? nextValues.values[0] : null;
      return this.focusItem(nextFocus && nextFocus.index || 1);
    }

    // Find the previous focused category
    const prevCategoryIndex = values.findIndex((category) => {
      return category.values.find((value) => value.index === focusedItem);
    });
    const prevFocusIndex = values[prevCategoryIndex].values.findIndex((a) => a.index === focusedItem);

    let nextCategory = prevCategoryIndex;
    let nextFocusIndex;

    // If down: increase index if possible
    if (direction === 'down') {
      const prevN = values[prevCategoryIndex].values.length;
      nextFocusIndex = Math.min(prevFocusIndex + 1, prevN - 1);
    }

    // If up: decrease index if possible
    if (direction === 'up') {
      // Focus on search if at the top
      if (prevFocusIndex === 0) {
        return this.handleDOMAction(this.inputRef, 'focus');
      }

      nextFocusIndex = prevFocusIndex - 1;
    }

    // If right: next category
    if (direction === 'right') {
      nextCategory = Math.min(prevCategoryIndex + 1, values.length - 1);
    }

    // If right: previous category
    if (direction === 'left') {
      nextCategory = Math.max(prevCategoryIndex - 1, 0);
    }

    // If left or right: try to keep the horizontal index
    if (direction === 'left' || direction === 'right') {
      this.setState({ focusedCat: nextCategory });
      nextFocusIndex = Math.min(prevFocusIndex, values[nextCategory].values.length - 1);
    }

    const nextFocus = values[nextCategory].values[nextFocusIndex].index;
    return this.focusItem(nextFocus);
  }

  handleClick = (address) => {
    this.props.onChange(null, address);
    this.handleClose();
  }

  handleFocus = () => {
    if (this.closing) {
      this.closing = false;
      return;
    }

    const { top, left } = this.refs.main.getBoundingClientRect();

    this.setState({ top, left, expanded: true, focusedItem: null, focusedCat: null }, () => {
      this.setState({ top: 0, left: 0 }, () => {
        window.setTimeout(() => {
          this.handleDOMAction(this.inputRef, 'focus');
        }, 250);
      });
    });
  }

  handleClose = () => {
    if (!this.refs.main) {
      return null;
    }

    const { top, left } = this.refs.main.getBoundingClientRect();
    this.setState({ top, left, expanded: false });
    this.closing = true;

    return this.handleDOMAction('inputAddress', 'focus');
  }

  /**
   * Filter the given values based on the given
   * filter
   */
  filterValues = (values = [], _filter = '') => {
    const filter = _filter.toLowerCase();

    return values
      // Remove empty accounts
      .filter((a) => a)
      .filter((account) => {
        const address = account.address.toLowerCase();
        const inAddress = address.includes(filter);

        if (!account.name || inAddress) {
          return inAddress;
        }

        const name = account.name.toLowerCase();
        return name.includes(filter);
      })
      .sort((accA, accB) => {
        const nameA = accA.name || accA.address;
        const nameB = accB.name || accB.address;

        return nameA.localeCompare(nameB);
      });
  }

  handleInputFocus = () => {
    this.setState({ focusedItem: null });
  }

  handleChange = (event = { target: {} }) => {
    const { value = '' } = event.target;
    let index = 0;

    const values = this.values
      .map((category) => {
        const filteredValues = this
          .filterValues(category.values, value)
          .map((value) => {
            index++;
            return { ...value, index: parseInt(index) };
          });

        return {
          label: category.label,
          values: filteredValues
        };
      })
      .filter((category) => category.values.length > 0);

    this.setState({
      values,
      focusedItem: null,
      inputValue: value
    });
  }
}

function mapStateToProps (state) {
  // const { accounts, contacts, contracts } = state.personal;
  const { accountsInfo } = state.personal;
  const { balances } = state.balances;

  return {
    // accounts,
    // contacts,
    // contracts,
    accountsInfo,
    balances
  };
}

export default connect(
  mapStateToProps
)(AddressSelector);

