import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, SelectField, MenuItem } from 'material-ui';

import InputText from '../../Inputs/Text';
import Loading from '../../Loading';

import { SIMPLE_TOKEN_ADDRESS_TYPE, SIMPLE_TLA_TYPE } from '../../Inputs/validation';

import styles from '../actions.css';

const initState = {
  queryKey: 'tla',
  form: {
    valid: false,
    value: ''
  }
};

export default class QueryAction extends Component {

  static propTypes = {
    show: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired,
    handleQueryToken: PropTypes.func.isRequired,

    loading: PropTypes.bool.isRequired,
    data: PropTypes.object,
    notFound: PropTypes.bool
  }

  state = initState;

  render () {
    return (
      <Dialog
        title={ 'search for a token' }
        open={ this.props.show }
        className={ styles.dialog }
        onRequestClose={ this.onClose }
        actions={ this.renderActions() } >
        { this.renderContent() }
      </Dialog>
    );
  }

  renderActions () {
    let { loading, data, notFound } = this.props;

    if (loading) {
      return (
        <FlatButton
          label='Loading...'
          primary
          disabled />
      );
    }

    let complete = data || notFound;

    if (complete) {
      return ([
        <FlatButton
          label='Close'
          primary
          onTouchTap={ this.onClose } />
      ]);
    }

    const isValid = this.state.form.valid;

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />,
      <FlatButton
        label='Query'
        primary
        disabled={ !isValid }
        onTouchTap={ this.onQuery } />
    ]);
  }

  renderContent () {
    let { loading, notFound, data } = this.props;

    if (loading) {
      return (
        <Loading size={ 1 } />
      );
    }

    if (notFound) {
      return (
        <p>No token has been found in the registry...</p>
      );
    }

    if (data) {
      return this.renderData();
    }

    return this.renderForm();
  }

  renderData () {
    let { data } = this.props;

    return (
      <pre>{ JSON.stringify(data, null, 4) }</pre>
    );
  }

  renderForm () {
    return (
      <div>
        <SelectField
          floatingLabelText='Select which field to query'
          fullWidth
          value={ this.state.queryKey }
          onChange={ this.onQueryKeyChange }>
          <MenuItem value='tla' label='TLA' primaryText='TLA' />
          <MenuItem value='address' label='Address' primaryText='Address' />
        </SelectField>

        {
          this.state.queryKey !== 'tla'
          ? (<InputText
            key={ 0 }

            floatingLabelText="Token's address"
            hintText='0xdeadbeef...'

            validationType={ SIMPLE_TOKEN_ADDRESS_TYPE }
            onChange={ this.onChange } />)
          : (<InputText
            key={ 1 }

            floatingLabelText="Token's TLA"
            hintText='GAV'

            validationType={ SIMPLE_TLA_TYPE }
            onChange={ this.onChange } />)
        }
      </div>
    );
  }

  onQueryKeyChange = (event, index, queryKey) => {
    this.setState({
      queryKey,
      form: { valid: false, value: '' }
    });
  }

  onChange = (valid, value) => {
    this.setState({
      form: {
        valid, value
      }
    });
  }

  onQuery = () => {
    let { queryKey, form } = this.state;

    this.props.handleQueryToken(queryKey, form.value);
  }

  onClose = () => {
    this.setState(initState);
    this.props.onClose();
  }

}
