import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import DropDownMenu from 'material-ui/DropDownMenu';
import MenuItem from 'material-ui/MenuItem';
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';
import renderAddress from '../ui/address.js';

import styles from './lookup.css';

const nullable = (type) => React.PropTypes.oneOfType([ React.PropTypes.oneOf([ null ]), type ]);

export default class Lookup extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    result: nullable(PropTypes.string.isRequired),
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired
  }

  state = { name: '', type: 'A' };

  render () {
    const name = this.state.name || this.props.name;
    const type = this.state.type || this.props.type;
    const { result, accounts, contacts } = this.props;

    let output = '';
    if (result) {
      if (type === 'A') {
        output = (<code>{ renderAddress(result, accounts, contacts, false) }</code>);
      } else {
        output = (<code>{ result }</code>);
      }
    }

    return (
      <Card className={ styles.lookup }>
        <CardHeader title={ 'Query the Registry' } />
        <div className={ styles.box }>
          <TextField
            className={ styles.spacing }
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <DropDownMenu
            className={ styles.spacing }
            value={ type }
            onChange={ this.onTypeChange }
          >
            <MenuItem value='A' primaryText='A – Ethereum address' />
            <MenuItem value='IMG' primaryText='IMG – hash of a picture in the blockchain' />
            <MenuItem value='CONTENT' primaryText='CONTENT – hash of a data in the blockchain' />
          </DropDownMenu>
          <RaisedButton
            className={ styles.spacing }
            label='Lookup'
            primary
            icon={ <SearchIcon /> }
            onClick={ this.onLookupClick }
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
  };
  onLookupClick = () => {
    this.props.actions.lookup(this.state.name, this.state.type);
  };
}
