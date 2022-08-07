import React from 'react';
import * as ReactDOM from 'react-dom/client';
import { init, tokenize } from '../pkg';

init();

const root = ReactDOM.createRoot(document.getElementById("main"));
const tks: any[] = tokenize("そして我々を選んだのかもしれない。");
root.render(
    <div>{
        tks.map((tk, i) => <div key={i}>{JSON.stringify(tk)}</div>)
    }</div>
);
