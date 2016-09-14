// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { IdentityIcon, Modal } from '../../ui';
import { newError } from '../../redux/actions';
import initShapeshift from '../../3rdparty/shapeshift';
import shapeshiftLogo from '../../images/shapeshift-logo.png';

import Options from './Options';
import styles from './fundAccount.css';

const shapeshift = initShapeshift();

const STAGE_NAMES = ['details', 'awaiting deposit', 'awaiting exchange', 'completed'];

class FundAccount extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0,
    coinSymbol: 'BTC',
    coins: []
  }

  componentDidMount () {
    this.retrieveCoins();
  }

  render () {
    const { stage } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ STAGE_NAMES }
        visible>
        { this.renderPage() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { address } = this.props;
    const { coins, stage } = this.state;

    const logo = (
      <a href='http://shapeshift.io' target='_blank' className={ styles.shapeshift }>
        <img src={ shapeshiftLogo } />
      </a>
    );
    const cancelBtn = (
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />
    );

    switch (stage) {
      case 0:
        return [
          logo,
          cancelBtn,
          <FlatButton
            disabled={ !coins.length }
            icon={ <IdentityIcon address={ address } button /> }
            label='Shift Funds'
            primary
            onTouchTap={ this.onShift } />
        ];
    }
  }

  renderPage () {
    const { coinSymbol, coins, stage } = this.state;

    switch (stage) {
      case 0:
        return (
          <Options
            coinSymbol={ coinSymbol }
            coins={ coins } />
        );
    }
  }

  nextStage = () => {
    const { stage } = this.state;

    this.setState({
      stage: stage + 1
    });
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  onShift = () => {
    this.nextStage();
  }

  retrieveCoins () {
    const { newError } = this.props;

    shapeshift
      .getCoins()
      .then((_coins) => {
        const coins = Object.values(_coins).filter((coin) => coin.status === 'available');

        this.setState({
          coins
        });
      })
      .catch((error) => {
        console.error('retrieveCoins', error);
        newError(new Error(`Failed to retrieve coins from ShapeShift.io: ${error.message}`));
      });
  }
}

function mapStateToProps (state) {
  return {};
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(FundAccount);
