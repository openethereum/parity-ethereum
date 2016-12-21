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
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';

import { nullableProptype } from '~/util/proptypes';

import Address from '../ui/address.js';
import renderImage from '../ui/image.js';
import recordTypeSelect from '../ui/record-type-select.js';

import { clear, lookup } from './actions';
import styles from './lookup.css';

class Lookup extends Component {

  static propTypes = {
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    result: nullableProptype(PropTypes.string.isRequired),

    clear: PropTypes.func.isRequired,
    lookup: PropTypes.func.isRequired
  }

  state = { name: '', type: 'A' };

  render () {
    const name = this.state.name || this.props.name;
    const type = this.state.type || this.props.type;
    const { result } = this.props;

    let output = '';
    if (result) {
      if (type === 'A') {
        output = (
          <code>
            <Address address={ result } shortenHash={ false } />
          </code>
        );
      } else if (type === 'IMG') {
        output = renderImage(result);
      } else if (type === 'CONTENT') {
        output = (<div>
          <code>{ result }</code>
          <p>This is most likely just the hash of the content you are looking for</p>
        </div>);
      } else {
        output = (<code>{ result }</code>);
      }
    }

    return (
      <Card className={ styles.lookup }>
        <CardHeader title={ 'Query the Registry' } />
        <div className={ styles.box }>
          <TextField
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          { recordTypeSelect(type, this.onTypeChange) }
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

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onTypeChange = (e, i, type) => {
    this.setState({ type });
    this.props.clear();
  };
  onLookupClick = () => {
    this.props.lookup(this.state.name, this.state.type);
  };
}

export default connect(
  // mapStateToProps
  (state) => state.lookup,
  // mapDispatchToProps
  (dispatch) => bindActionCreators({ clear, lookup }, dispatch)
)(Lookup);
