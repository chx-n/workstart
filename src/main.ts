import { mount } from 'svelte';

import './styles.css';
import App from './App.svelte';
import Settings from './Settings.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('Missing application mount point');
}

const Component = new URLSearchParams(window.location.search).get('window') === 'settings' ? Settings : App;
const app = mount(Component, { target });

export default app;
