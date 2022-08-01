import React from 'react';
import ReactDOM from 'react-dom';
import { greet } from '../pkg';

ReactDOM.render(
    <p>{greet("world")}</p>,
    document.body
);
