// no need for react since not using JSX
import { Component, PropTypes } from 'react';

export default class Web3Provider extends Component {

  static childContextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    web3: PropTypes.object.isRequired,
    children: PropTypes.element
  };

  getChildContext () {
    return {
      web3: this.props.web3
    };
  }

  render () {
    return this.props.children;
  }

}
