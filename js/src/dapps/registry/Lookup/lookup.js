import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';
import renderAddress from '../ui/address.js';

import styles from './lookup.css';

export default class Lookup extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    name: PropTypes.string.isRequired,
    entry: PropTypes.string.isRequired,
    result: PropTypes.string.isRequired,
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired
  }

  state = { name: '', entry: 'A' };

  render () {
    const name = this.state.name || this.props.name;
    const entry = this.state.entry || this.props.entry;
    const { result, accounts, contacts } = this.props;

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
          <TextField
            className={ styles.spacing }
            hintText='entry'
            value={ entry }
            onChange={ this.onKeyChange }
          />
          <RaisedButton
            className={ styles.spacing }
            label='Lookup'
            primary
            icon={ <SearchIcon /> }
            onClick={ this.onLookupClick }
          />
        </div>
        <CardText>
          { result
            ? (<code>{ renderAddress(result, accounts, contacts, false) }</code>)
            : ''
          }
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onKeyChange = (e) => {
    this.setState({ entry: e.target.value });
  };
  onLookupClick = () => {
    this.props.actions.lookup(this.state.name, this.state.entry);
  };
}
