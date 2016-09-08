import React, { Component, PropTypes } from 'react';
import TextField from 'material-ui/TextField';
import FlatButton from 'material-ui/FlatButton';

import styles from './lookup.css';

export default class Lookup extends Component {

  static propTypes = {
    actions: PropTypes.object,
    lookup: PropTypes.object
  }

  state = { name: '', key: '' };

  render () {
    const { name, key } = this.state
    const props = this.props.lookup
    return (

      <div className={ styles.lookup }>
        <TextField
          hintText='name'
          value={ name || props.name || '' }
          onChange={ this.onNameChange }
        />
        <TextField
          hintText='key'
          value={ key || props.key || '' }
          onChange={ this.onKeyChange }
        />
        <FlatButton
          label='Lookup'
          primary
          onClick={ this.onLookupClick }
        />
        <div className={ styles.results }>
          { this.props.lookup.result || '' }
        </div>
      </div>
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
