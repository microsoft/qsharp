import {
  JupyterFrontEnd,
  JupyterFrontEndPlugin
} from '@jupyterlab/application';

/**
 * Initialization data for the qsharp_jupyterlab extension.
 */
const plugin: JupyterFrontEndPlugin<void> = {
  id: 'qsharp_jupyterlab:plugin',
  autoStart: true,
  activate: (app: JupyterFrontEnd) => {
    console.log('JupyterLab extension qsharp_jupyterlab is activated!');
  }
};

export default plugin;
