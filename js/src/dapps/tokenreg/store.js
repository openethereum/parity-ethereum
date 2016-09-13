import { createStore, applyMiddleware } from 'redux';
import thunk from 'redux-thunk';

import reducer from './reducers';

export default createStore(reducer, applyMiddleware(thunk));
