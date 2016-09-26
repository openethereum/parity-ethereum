var express = require('express');
var proxy = require('http-proxy-middleware');

var app = express();

app.use(express.static('build'));

app.use('/api/*', proxy({
  target: 'http://127.0.0.1:8080',
  changeOrigin: true
}));

app.use('/rpc/*', proxy({
  target: 'http://localhost:8080',
  changeOrigin: true
}));

app.listen(3000);
