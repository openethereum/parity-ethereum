/* @flow */
import React, { Component } from 'react';

/** Stylesheets **/
import './Switch.css';

type Props = {|
  defaultValue: bool
|}

type State = {|
  state: bool,
  position: Object
|}

class Switch extends Component {
  props: Props;
  state: State = {
    state: false,
    position: { transform: 'translateX(-50px)' }
  }

  componentWillMount () {
    const { defaultValue } = this.props;

    if (defaultValue) {
      this.setState({
        state: true,
        position: { transform: 'translateX(0px)' }
      });
    }
  }

  switchClick = () => {
    const { state } = this.state;

    if (state) {
      this.setState({
        state: false,
        position: { transform: 'translateX(-50px)' }
      });
    } else {
      this.setState({
        state: true,
        position: { transform: 'translateX(0px)' }
      });
    }
  }

  render () {
    const { position } = this.state;

    return (
      <div className='Switch'>

        <div className='switch-body-button' style={ position }>
          <div id='switch-button'>
            <div id='switch-center-button' />
          </div>
        </div>

        <div className='switch-body' onClick={ this.switchClick }>
          <div id='clicker' style={ position }>
            <div id='switch-light' />
            <div id='switch-right-button' />
          </div>
        </div>

      </div>
    );
  }
}

export default Switch;
