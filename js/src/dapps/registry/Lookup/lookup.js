import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';
import renderAddress from '../ui/address.js';
import renderImage from '../ui/image.js';

import recordTypeSelect from '../ui/record-type-select.js';
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
      } else if (type === 'IMG') {
        output = renderImage(result);
      } else if (type === 'CONTENT') {
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
            className={ styles.spacing }
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          { recordTypeSelect(type, this.onTypeChange, styles.spacing) }
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
    this.props.actions.clear();
  };
  onLookupClick = () => {
    this.props.actions.lookup(this.state.name, this.state.type);
  };
}
