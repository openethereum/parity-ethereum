import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import SaveIcon from 'material-ui/svg-icons/content/save';

import recordTypeSelect from '../ui/record-type-select.js';
import styles from './records.css';

export default class Records extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    hasAccount: PropTypes.bool.isRequired,
    pending: PropTypes.bool.isRequired,
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    value: PropTypes.string.isRequired
  }

  state = { name: '', type: 'A', value: '' };

  render () {
    const { hasAccount, pending } = this.props;
    const name = this.state.name || this.props.name;
    const type = this.state.type || this.props.type;
    const value = this.state.value || this.props.value;

    return (
      <Card className={ styles.records }>
        <CardHeader title={ 'Manage Entries of a Name' } />
        <CardText>
          { !hasAccount
            ? (<p className={ styles.noSpacing }>Please select an account first.</p>)
            : (<p className={ styles.noSpacing }>You can only modify entries of names that you previously registered.</p>)
          }
          <TextField
            className={ styles.spacing }
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          { recordTypeSelect(type, this.onTypeChange, styles.spacing) }
          <TextField
            className={ styles.spacing }
            hintText='value'
            value={ value }
            onChange={ this.onValueChange }
          />
          <RaisedButton
            disabled={ !hasAccount || pending }
            className={ styles.spacing }
            label='Save'
            primary
            icon={ <SaveIcon /> }
            onClick={ this.onSaveClick }
          />
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onTypeChange = (e, i, type) => {
    this.setState({ type });
  };
  onValueChange = (e) => {
    this.setState({ value: e.target.value });
  };
  onSaveClick = () => {
    const { name, type, value } = this.state;
    this.props.actions.update(name, type, value);
  };
}
