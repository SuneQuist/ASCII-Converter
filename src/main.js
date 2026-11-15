const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const dropzone = document.getElementById('drop-zone');
const loader = document.querySelector('.loader');
const screen = dropzone.querySelector('img');

const floater = document.querySelector('.floater'); // Container for resizers 
const resize_width = document.getElementById('resize-width');
const resize_height = document.getElementById('resize-height');

var processes = 0;

function activate_drop() {
  listen('tauri://drag-over', () => { dropzone.classList.add('drag-over'); });
  listen('tauri://drag-leave', () => { dropzone.classList.remove('drag-over'); });
  
  listen('tauri://drag-drop', (event) => {
    dropzone.classList.remove('drag-over');
    const paths = event.payload || [];

    if (paths?.paths && paths.paths.length <= 0) { return }

    const allowed_extensions = ['png', 'jpg', 'webp', 'pdf', 'mp4', 'mov', 'mkv', 'avi', 'mp3', 'aac', 'wav', 'flac'];
    paths.paths.forEach(path => {
      const ext = (path.split('.')).pop();
      if (allowed_extensions.includes(ext.toLowerCase())) {
        let width = resize_width.valueAsNumber;
        let height = resize_height.valueAsNumber

        invoke('to_ascii', { 
          path: path, 
          width: !isNaN(width) && width > 0 ? width : 0, 
          height: !isNaN(height) && height > 0 ? height : 0
        });
        
        processes++;
        loader.classList.add('-visible');
        screen.classList.remove('-visible');
      }
    });
  });

  listen("process-done", (event) => {
    processes--;

    if (processes <= 0) {
      loader.classList.remove('-visible');
      screen.classList.add('-visible');
      processes = 0;
      return;
    }
  })
}

activate_drop();