import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import SearchIcon from 'material-ui/svg-icons/action/search';

import styles from './lookup.css';

export default class Lookup extends Component {

  static propTypes = {
    actions: PropTypes.object,
    lookup: PropTypes.object
  }

  state = { name: '', key: 'A' };

  render () {
    const { name, key } = this.state;
    const props = this.props.lookup;
    return (

      <Card className={ styles.lookup }>
        <CardHeader title={ 'Query the Registry' } />
        <div className={ styles.box }>
          <TextField
            className={ styles.spacing }
            hintText='name'
            value={ name || props.name || '' }
            onChange={ this.onNameChange }
          />
          <TextField
            className={ styles.spacing }
            hintText='key'
            value={ key || props.key }
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
          <code>{ this.props.lookup.result || '' }</code>
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onKeyChange = (e) => {
    this.setState({ key: e.target.value });
  };
  onLookupClick = () => {
    this.props.actions.lookup(this.state.name, this.state.key);
  };
}
