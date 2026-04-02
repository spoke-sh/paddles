import React from 'react';
import ReactDOM from 'react-dom/client';

import { RuntimeApp } from './runtime-app';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RuntimeApp />
  </React.StrictMode>
);
