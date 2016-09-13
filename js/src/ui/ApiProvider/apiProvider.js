import { Component, PropTypes } from 'react';

export default class ApiProvider extends Component {
  static propTypes = {
    api: PropTypes.object.isRequired,
    children: PropTypes.node.isRequired
  }

  static childContextTypes = {
    api: PropTypes.object
  }

  render () {
    const { children } = this.props;

    return children;
  }

  getChildContext () {
    const { api } = this.props;

    return {
      api
    };
  }
}
