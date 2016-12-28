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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import DropDownMenu from 'material-ui/DropDownMenu';
import MenuItem from 'material-ui/MenuItem';
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';

import { nullableProptype } from '~/util/proptypes';

import Address from '../ui/address.js';
import renderImage from '../ui/image.js';

import { clear, lookup, reverseLookup } from './actions';
import styles from './lookup.css';

class Lookup extends Component {

  static propTypes = {
    result: nullableProptype(PropTypes.string.isRequired),

    clear: PropTypes.func.isRequired,
    lookup: PropTypes.func.isRequired,
    reverseLookup: PropTypes.func.isRequired
  }

  state = {
    input: '', type: 'A'
  };

  render () {
    const { input, type } = this.state;
    const { result } = this.props;

    let output = '';
    if (result) {
      if (type === 'A') {
        output = (
          <code>
            <Address
              address={ result }
              shortenHash={ false }
            />
          </code>
        );
      } else if (type === 'IMG') {
        output = renderImage(result);
      } else if (type === 'CONTENT') {
        output = (
          <div>
            <code>{ result }</code>
            <p>Keep in mind that this is most likely the hash of the content you are looking for.</p>
          </div>
        );
      } else {
        output = (
          <code>{ result }</code>
        );
      }
    }

    return (
      <Card className={ styles.lookup }>
        <CardHeader title={ 'Query the Registry' } />
        <div className={ styles.box }>
          <TextField
            hintText={ type === 'reverse' ? 'address' : 'name' }
            value={ input }
            onChange={ this.onInputChange }
          />
          <DropDownMenu
            value={ type }
            onChange={ this.onTypeChange }
          >
            <MenuItem value='A' primaryText='A – Ethereum address' />
            <MenuItem value='IMG' primaryText='IMG – hash of a picture in the blockchain' />
            <MenuItem value='CONTENT' primaryText='CONTENT – hash of a data in the blockchain' />
            <MenuItem value='reverse' primaryText='reverse – find a name for an address' />
          </DropDownMenu>
          <RaisedButton
            label='Lookup'
            primary
            icon={ <SearchIcon /> }
            onTouchTap={ this.onLookupClick }
          />
        </div>
        <CardText>{ output }</CardText>
      </Card>
    );
  }

  onInputChange = (e) => {
    this.setState({ input: e.target.value });
  };

  onTypeChange = (e, i, type) => {
    this.setState({ type });
    this.props.clear();
  };

  onLookupClick = () => {
    const { input, type } = this.state;

    if (type === 'reverse') {
      this.props.reverseLookup(input);
    } else {
      this.props.lookup(input, type);
    }
  };
}

const mapStateToProps = (state) => state.lookup;
const mapDispatchToProps = (dispatch) =>
  bindActionCreators({
    clear, lookup, reverseLookup
  }, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(Lookup);
