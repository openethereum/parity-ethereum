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
    const self = this;
    const onNameChange = (e) => {
      self.setState({ name: e.target.value });
    };
    const onKeyChange = (e) => {
      self.setState({ key: e.target.value });
    };
    const onLookupClick = () => {
      self.props.actions.lookup(self.state.name, self.state.key);
    };

    return (
      <div className={ styles.lookup }>
        <TextField
          hintText='name'
          value={ this.state.name || this.props.lookup.name || '' }
          onChange={ onNameChange }
        />
        <TextField
          hintText='key'
          value={ this.state.key || this.props.lookup.key || '' }
          onChange={ onKeyChange }
        />
        <FlatButton
          label='Lookup'
          primary
          onClick={ onLookupClick }
        />
        <div className={ styles.results }>
          { this.props.lookup.result || '' }
        </div>
      </div>
    );
  }
}
