import React, { Component, PropTypes } from 'react';
import { Dialog, RaisedButton, FlatButton, SelectField, MenuItem } from 'material-ui';
import AddIcon from 'material-ui/svg-icons/content/add';

import InputText from '../../Inputs/Text';

import styles from './token.css';

import { metaDataKeys } from '../../constants';

const initState = {
  showDialog: false,
  complete: false,
  metaKeyIndex: 0,

  form: {
    valid: false,
    value: ''
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

    return (<div className={ styles['add-meta'] }>
      <RaisedButton
        label='Add Meta-Data'
        icon={ <AddIcon /> }
        primary
        fullWidth
        onTouchTap={ this.onShowDialog } />

      <Dialog
        title='add meta data'
        open={ this.state.showDialog }
        modal={ this.state.complete }
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

    const isValid = this.state.form.valid;

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
    if (metaDataKeys[this.state.metaKeyIndex].value === 'IMG') {
      return (<div>
        <p>
        Your transactions has been posted.
        Two transactions are needed to add an Image.
        Please visit the Parity Signer to authenticate the transfer.</p>
      </div>);
    }
    return (<div>
      <p>Your transaction has been posted. Please visit the Parity Signer to authenticate the transfer.</p>
    </div>);
  }

  renderForm () {
    let selectedMeta = metaDataKeys[this.state.metaKeyIndex];

    return (
      <div>
        <SelectField
          floatingLabelText='Choose the meta-data to add'
          fullWidth
          value={ this.state.metaKeyIndex }
          onChange={ this.onMetaKeyChange }>

          { this.renderMetaKeyItems() }

        </SelectField>

        <InputText
          key={ selectedMeta.value }
          floatingLabelText={ `${selectedMeta.label} value` }
          hintText={ `The value of the ${selectedMeta.label.toLowerCase()} meta-data` }

          validationType={ selectedMeta.validation }
          onChange={ this.onChange } />
      </div>
    );
  }

  renderMetaKeyItems () {
    return metaDataKeys.map((key, index) => (
      <MenuItem
        value={ index }
        key={ index }
        label={ key.label } primaryText={ key.label } />
    ));
  }

  onShowDialog () {
    this.setState({ showDialog: true });
  }

  onClose () {
    this.setState(initState);
  }

  onAdd (index) {
    let keyIndex = this.state.metaKeyIndex;
    let key = metaDataKeys[keyIndex].value;

    this.props.handleAddMeta(
      index,
      key,
      this.state.form.value
    );

    this.setState({ complete: true });
  }

  onChange = (valid, value) => {
    this.setState({
      form: {
        valid, value
      }
    });
  }

  onMetaKeyChange = (event, metaKeyIndex) => {
    this.setState({ metaKeyIndex, form: {
      ...[this.state.form],
      valid: false,
      value: ''
    } });
  }

}
