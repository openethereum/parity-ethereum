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
import ReactDOM from 'react-dom';
import { connect } from 'react-redux';
import keycode, { codes } from 'keycode';
import { FormattedMessage } from 'react-intl';
import { observer } from 'mobx-react';

import TextFieldUnderline from 'material-ui/TextField/TextFieldUnderline';

import apiutil from '@parity/api/lib/util';
import AccountCard from '~/ui/AccountCard';
import CopyToClipboard from '~/ui/CopyToClipboard';
import InputAddress from '~/ui/Form/InputAddress';
import Loading from '~/ui/Loading';
import Portal from '~/ui/Portal';
import { nodeOrStringProptype } from '~/util/proptypes';
import { validateAddress } from '~/util/validation';
import { toString } from '~/util/messages';

import AddressSelectStore from './addressSelectStore';
import styles from './addressSelect.css';

const BOTTOM_BORDER_STYLE = { borderBottom: 'solid 3px' };

// Current Form ID
let currentId = 1;

@observer
class AddressSelect extends Component {
  static contextTypes = {
    intl: React.PropTypes.object.isRequired,
    api: PropTypes.object.isRequired,
    muiTheme: PropTypes.object.isRequired
  };

  static propTypes = {
    // Required props
    onChange: PropTypes.func.isRequired,

    // Redux props
    accountsInfo: PropTypes.object,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    reverse: PropTypes.object,

    // Optional props
    allowCopy: PropTypes.bool,
    allowInput: PropTypes.bool,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: nodeOrStringProptype(),
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    readOnly: PropTypes.bool,
    value: nodeOrStringProptype()
  };

  static defaultProps = {
    value: ''
  };

  store = new AddressSelectStore(this.context.api);

  state = {
    expanded: false,
    focused: false,
    focusedCat: null,
    focusedItem: null,
    inputFocused: false,
    inputValue: ''
  };

  componentWillMount () {
    this.setValues();
  }

  componentWillReceiveProps (nextProps) {
    if (this.store.values && this.store.values.length > 0) {
      return;
    }

    this.setValues(nextProps);
  }

  setValues (props = this.props) {
    this.store.setValues(props);
  }

  render () {
    const input = this.renderInput();
    const content = this.renderContent();

    return (
      <div className={ styles.main }>
        { input }
        { content }
      </div>
    );
  }

  renderInput () {
    const { focused } = this.state;
    const { accountsInfo, allowCopy, className, disabled, error, hint, label, readOnly, value } = this.props;

    const input = (
      <InputAddress
        accountsInfo={ accountsInfo }
        allowCopy={ (disabled || readOnly) && allowCopy ? allowCopy : false }
        className={ className }
        disabled={ disabled || readOnly }
        error={ error }
        hint={ hint }
        focused={ focused }
        label={ label }
        readOnly
        tabIndex={ -1 }
        text
        value={ value }
      />
    );

    if (disabled || readOnly) {
      return input;
    }

    return (
      <div className={ styles.inputAddressContainer }>
        { this.renderCopyButton() }
        <div
          className={ styles.inputAddress }
          onBlur={ this.handleMainBlur }
          onClick={ this.handleFocus }
          onFocus={ this.handleMainFocus }
          onKeyDown={ this.handleInputAddresKeydown }
          ref='inputAddress'
          tabIndex={ 0 }
        >
          { input }
        </div>
      </div>
    );
  }

  renderCopyButton () {
    const { allowCopy, value } = this.props;

    if (!allowCopy) {
      return null;
    }

    const text = typeof allowCopy === 'string'
      ? allowCopy
      : value.toString();

    return (
      <div className={ styles.copy }>
        <CopyToClipboard data={ text } />
      </div>
    );
  }

  renderContent () {
    const { muiTheme } = this.context;
    const { hint, disabled, label, readOnly } = this.props;
    const { expanded, inputFocused } = this.state;

    if (disabled || readOnly) {
      return null;
    }

    const id = `addressSelect_${++currentId}`;
    const ilHint = toString(this.context, hint);

    return (
      <Portal
        className={ styles.inputContainer }
        isChildModal
        onClick={ this.handleClose }
        onClose={ this.handleClose }
        onKeyDown={ this.handleKeyDown }
        open={ expanded }
        title={
          <div className={ styles.title }>
            <label className={ styles.label } htmlFor={ id }>
              { label }
            </label>
            <div className={ styles.outerInput }>
              <input
                id={ id }
                className={ styles.input }
                placeholder={ ilHint }
                onBlur={ this.handleInputBlur }
                onClick={ this.stopEvent }
                onFocus={ this.handleInputFocus }
                onChange={ this.handleChange }
                ref={ this.setInputRef }
              />
              { this.renderLoader() }
            </div>

            <div className={ styles.underline }>
              <TextFieldUnderline
                focus={ inputFocused }
                focusStyle={ BOTTOM_BORDER_STYLE }
                muiTheme={ muiTheme }
                style={ BOTTOM_BORDER_STYLE }
              />
            </div>
          </div>
        }
      >
        { this.renderCurrentInput() }
        { this.renderRegistryValues() }
        { this.renderAccounts() }
      </Portal>
    );
  }

  renderLoader () {
    if (!this.store.loading) {
      return null;
    }

    return (
      <Loading
        className={ styles.loader }
        size={ 0.5 }
      />
    );
  }

  renderCurrentInput () {
    const { inputValue } = this.state;

    if (!this.props.allowInput || !inputValue) {
      return null;
    }

    const { address, addressError } = validateAddress(inputValue);
    const { registryValues } = this.store;

    if (addressError || registryValues.length > 0) {
      return null;
    }

    return (
      <div className={ styles.container }>
        { this.renderAccountCard({ address, index: 'currentInput_0' }) }
      </div>
    );
  }

  renderRegistryValues () {
    const { registryValues } = this.store;

    if (registryValues.length === 0) {
      return null;
    }

    const accounts = registryValues
      .map((registryValue, index) => {
        const account = { ...registryValue, index: `${registryValue.address}_${index}` };

        return this.renderAccountCard(account);
      });

    return (
      <div className={ styles.container }>
        { accounts }
      </div>
    );
  }

  renderAccounts () {
    const { values } = this.store;

    if (values.length === 0) {
      return (
        <div className={ styles.categories }>
          <div className={ styles.empty }>
            <FormattedMessage
              id='addressSelect.noAccount'
              defaultMessage='No account matches this query...'
            />
          </div>
        </div>
      );
    }

    const categories = values.map((category, index) => {
      return this.renderCategory(category, index);
    });

    return (
      <div className={ styles.categories }>
        { categories }
      </div>
    );
  }

  renderCategory (category, index) {
    const { label, key, values = [] } = category;
    let content;

    if (values.length === 0) {
      content = (
        <p>
          <FormattedMessage
            id='addressSelect.noAccount'
            defaultMessage='No account matches this query...'
          />
        </p>
      );
    } else {
      const cards = values
        .map((account) => this.renderAccountCard(account));

      content = (
        <div className={ styles.cards }>
          <div>{ cards }</div>
        </div>
      );
    }

    return (
      <div className={ styles.category } key={ `${key}_${index}` }>
        <div className={ styles.title }>
          <h3>{ label }</h3>
        </div>
        { content }
      </div>
    );
  }

  renderAccountCard (_account) {
    const { accountsInfo } = this.props;
    const { address, index = null } = _account;

    const account = {
      ...accountsInfo[address],
      ..._account
    };

    return (
      <AccountCard
        account={ account }
        className={ styles.account }
        key={ `account_${index}` }
        onClick={ this.handleClick }
        onFocus={ this.focusItem }
        ref={ `account_${index}` }
      />
    );
  }

  setInputRef = (refId) => {
    this.inputRef = refId;
  }

  validateCustomInput = () => {
    const { allowInput } = this.props;
    const { inputValue } = this.state;
    const { values } = this.store;

    // If input is HEX and allowInput === true, send it
    if (allowInput && inputValue && /^(0x)?([0-9a-f])+$/i.test(inputValue)) {
      return this.handleClick(inputValue);
    }

    // If only one value, select it
    if (values.reduce((cur, cat) => cur + cat.values.length, 0) === 1) {
      const value = values.find((cat) => cat.values.length > 0).values[0];

      return this.handleClick(value.address);
    }
  }

  stopEvent = (event) => {
    event.stopPropagation();
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

    if (event.ctrlKey) {
      return event;
    }

    switch (codeName) {
      case 'enter':
        const index = this.state.focusedItem;

        if (!index) {
          return this.validateCustomInput();
        }

        return this.handleDOMAction(`account_${index}`, 'click');

      case 'left':
      case 'right':
      case 'up':
      case 'down':
        return this.handleNavigation(codeName, event);

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

  handleNavigation = (direction, event) => {
    const { focusedItem, focusedCat } = this.state;
    const { values } = this.store;

    // Don't do anything if no values
    if (values.reduce((cur, cat) => cur + cat.values.length, 0) === 0) {
      return event;
    }

    // Focus on the first element if none selected yet if going down
    if (!focusedItem) {
      if (direction !== 'down') {
        return event;
      }

      event.preventDefault();

      const firstCat = values.findIndex((cat) => cat.values.length > 0);
      const nextCat = focusedCat && values[focusedCat].values.length > 0
        ? focusedCat
        : firstCat;

      const nextValues = values[nextCat];
      const nextFocus = nextValues ? nextValues.values[0] : null;

      return this.focusItem(nextFocus && nextFocus.index || 1);
    }

    event.preventDefault();

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
      const categoryShift = values
        .slice(prevCategoryIndex + 1, values.length)
        .findIndex((cat) => cat.values.length > 0) + 1;

      nextCategory = Math.min(prevCategoryIndex + categoryShift, values.length - 1);
    }

    // If right: previous category
    if (direction === 'left') {
      const categoryShift = values
        .slice(0, prevCategoryIndex)
        .reverse()
        .findIndex((cat) => cat.values.length > 0) + 1;

      nextCategory = Math.max(prevCategoryIndex - categoryShift, 0);
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
    // Don't do anything if it's only text-selection
    if (window.getSelection && window.getSelection().type === 'Range') {
      return;
    }

    this.props.onChange(null, address);
    this.handleClose();
  }

  handleMainBlur = () => {
    if (this.props.readOnly) {
      return;
    }

    if (window.document.hasFocus() && !this.state.expanded) {
      this.closing = false;
      this.setState({ focused: false });
    }
  }

  handleMainFocus = () => {
    if (this.state.focused || this.props.readOnly) {
      return;
    }

    this.setState({ focused: true }, () => {
      if (this.closing) {
        this.closing = false;
        return;
      }

      this.handleFocus();
    });
  }

  handleFocus = () => {
    const { disabled, readOnly } = this.props;

    if (disabled || readOnly) {
      return;
    }

    this.setState({ expanded: true, focusedItem: null, focusedCat: null }, () => {
      window.setTimeout(() => {
        this.handleDOMAction(this.inputRef, 'focus');
      });
    });
  }

  handleClose = () => {
    this.closing = true;

    if (this.refs.inputAddress) {
      this.handleDOMAction('inputAddress', 'focus');
    }

    this.store.resetRegistryValues();
    this.store.handleChange('');

    this.setState({
      expanded: false,
      focusedItem: null,
      inputValue: ''
    });
  }

  handleInputBlur = () => {
    this.setState({ inputFocused: false });
  }

  handleInputFocus = () => {
    this.setState({ focusedItem: null, inputFocused: true });
  }

  handleChange = (event = { target: {} }) => {
    const { value = '' } = event.target;

    this.store.handleChange(value);

    this.setState({
      focusedItem: null,
      inputValue: value
    });

    if (apiutil.isAddressValid(value)) {
      this.handleClick(value);
    }
  }
}

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;
  const { reverse } = state.registry;

  return {
    accountsInfo,
    reverse
  };
}

export default connect(
  mapStateToProps
)(AddressSelect);
