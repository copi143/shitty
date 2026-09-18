#!/usr/bin/env python3
import os
from flask import Flask, send_file

CWD = os.path.join(os.getcwd(), 'www')

os.chdir(CWD)

app = Flask(__name__, root_path=CWD)


@app.route('/', defaults={'filename': ''}, methods=['GET'])
@app.route('/<path:filename>', methods=['GET'])
def serve_file(filename):
  if filename == '':
    filename = 'index.html'
    if os.path.isfile(filename):
      return send_file(filename)
    return 'Not Found', 404
  if os.path.isdir(filename):
    filename = os.path.join(filename, 'index.html')
    if os.path.exists(filename):
      return send_file(filename)
    return 'Not Found', 404
  if os.path.isfile(filename):
    return send_file(filename)
  if os.path.isfile(filename + '.html'):
    return send_file(filename + '.html')
  return 'Not Found', 404


app.run(host='127.0.0.1', port=8000)
