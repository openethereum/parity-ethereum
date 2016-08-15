import React, { Component, PropTypes } from 'react';

import ContentCreate from 'material-ui/svg-icons/content/create';

import { FundAccount, Transfer } from '../../modals';

import Balances from '../../Balances';
import Container, { Title } from '../../Container';
import Form, { FormWrap, Input, InputInline } from '../../Form';
import IdentityIcon from '../../IdentityIcon';

import Actions from './actions';

import styles from './style.css';

export default class Account extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    params: PropTypes.object
  }

  state = {
    name: 'Unnamed',
    fundDialog: false,
    transferDialog: false
  }

  componentWillMount () {
    this.retrieveMeta();
  }

  render () {
    const address = this.props.params.address;
    const title = (
      <span>
        <span>{ this.state.name }</span>
        <ContentCreate
          className={ styles.editicon }
          color='rgb(0, 151, 167)' />
      </span>
    );

    return (
      <div>
        <FundAccount
          address={ address }
          onClose={ this.onFundAccountClose }
          visible={ this.state.fundDialog } />
        <Transfer
          address={ address }
          onClose={ this.onTransferClose }
          visible={ this.state.transferDialog } />
        <Actions
          onFundAccount={ this.onFundAccountClick }
          onTransfer={ this.onTransferClick } />
        <Container>
          <IdentityIcon
            address={ address } />
          <Form>
            <FormWrap>
              <InputInline
                label='account name'
                hint='a descriptive name for the account'
                value={ this.state.name }
                static={ <Title title={ title } /> }
                onChange={ this.onEditName } />
            </FormWrap>
            <FormWrap>
              <Input
                disabled
                label='account address'
                hint='the account network address'
                value={ address } />
            </FormWrap>
          </Form>
          <Balances
            address={ address } />
        </Container>
      </div>
    );
  }

  onFundAccountClick = () => {
    this.setState({ fundDialog: !this.state.fundDialog });
  }

  onFundAccountClose = () => {
    this.onFundAccountClick();
  }

  onTransferClick = () => {
    this.setState({ transferDialog: !this.state.transferDialog });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }

  onEditName = (event) => {
    const api = this.context.api;
    const name = event.target.value;

    this.setState({
      name: name
    }, () => {
      api.personal.setAccountName(this.props.params.address, name);
    });
  }

  retrieveMeta () {
    this.context.api.personal
      .accountsInfo()
      .then((infos) => {
        const info = infos[this.props.params.address];
        this.setState({
          name: info.name,
          uuid: info.uuid,
          meta: info.meta
        });
      });
  }
}
