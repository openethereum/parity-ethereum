import React, { Component, PropTypes } from 'react';
import { Dialog, RaisedButton, FlatButton, TextField } from 'material-ui';
import AddIcon from 'material-ui/svg-icons/content/add';

import { STRING_TYPE, validate } from '../../Actions/validation';

import styles from './token.css';

const defaultField = { error: null, value: '', valid: false };
const initState = {
  showDialog: false,
  complete: false,
  fields: {
    key: { ...defaultField, type: STRING_TYPE },
    value: { ...defaultField, type: STRING_TYPE }
  }
};

export default class AddMeta extends Component {
  static propTypes = {
    isTokenOwner: PropTypes.bool,
    handleAddMeta: PropTypes.func,
    index: PropTypes.number
  };

  state = initState;

  constructor (...args) {
    super(...args);

    this.onShowDialog = this.onShowDialog.bind(this);
    this.onClose = this.onClose.bind(this);
    this.onAdd = this.onAdd.bind(this, this.props.index);
  }

  render () {
    if (!this.props.isTokenOwner) return null;

    return (<div>
      <RaisedButton
        className={ styles['add-meta'] }
        label='Add Meta'
        icon={ <AddIcon /> }
        primary
        fullWidth
        onTouchTap={ this.onShowDialog } />

      <Dialog
        title='add meta data'
        open={ this.state.showDialog }
        modal
        className={ styles.dialog }
        onRequestClose={ this.onClose }
        actions={ this.renderActions() } >
        { this.renderContent() }
      </Dialog>
    </div>);
  }

  renderActions () {
    const { complete } = this.state;

    if (complete) {
      return (
        <FlatButton
          label='Done'
          primary
          onTouchTap={ this.onClose } />
      );
    }

    const isValid = this.isValid();

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />,
      <FlatButton
        label='Add'
        primary
        disabled={ !isValid }
        onTouchTap={ this.onAdd } />
    ]);
  }

  renderContent () {
    let { complete } = this.state;

    if (complete) return this.renderComplete();
    return this.renderForm();
  }

  renderComplete () {
    return (<div>
      <p>Your transaction has been posted. Please visit the Parity Signer to authenticate the transfer.</p>
    </div>);
  }

  renderForm () {
    const { fields } = this.state;

    let onChangeKey = this.onChange.bind(this, 'key');
    let onChangeValue = this.onChange.bind(this, 'value');

    return (
      <div>
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Meta Key'
          fullWidth
          hintText='The key of the meta-data'
          errorText={ fields.key.error }
          onChange={ onChangeKey } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Meta Value'
          fullWidth
          hintText='The value of the meta-data'
          errorText={ fields.value.error }
          onChange={ onChangeValue } />
      </div>
    );
  }

  isValid () {
    const { fields } = this.state;

    return Object.keys(fields)
      .map(key => fields[key].valid)
      .reduce((current, fieldValid) => {
        return current && fieldValid;
      }, true);
  }

  onShowDialog () {
    this.setState({ showDialog: true });
  }

  onClose () {
    this.setState(initState);
  }

  onAdd (index) {
    this.props.handleAddMeta(
      index,
      this.state.fields.key.value,
      this.state.fields.value.value
    );

    this.setState({ complete: true });
  }

  onChange (fieldKey, event) {
    const value = event.target.value;

    let fields = this.state.fields;
    let fieldState = fields[fieldKey];
    let validation = validate(value, fieldState.type);

    let newFieldState = {
      ...fieldState,
      ...validation
    };

    newFieldState.value = (validation.value !== undefined)
      ? validation.value
      : value;

    this.setState({
      fields: {
        ...fields,
        [fieldKey]: newFieldState
      }
    });
  }

}
