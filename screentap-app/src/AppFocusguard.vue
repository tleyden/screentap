<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Focusguard from "./components/Focusguard.vue";


import { listen } from '@tauri-apps/api/event';

// Listen for the custom event emitted from Rust
/**
 *                 let event_name = "update-screenshot-event"; // The event name to emit
                let payload = serde_json::json!({
                    "screenshot_id": screenshot_id
                });
 * 
 */
listen('my-custom-event', (event) => {
  console.log('Event received from Rust:', event.payload);

  alert('Got event!!');

  // add a new button to the document
  const button = document.createElement('button');
  button.textContent = 'Click me';
  document.body.appendChild(button);
  button.addEventListener('click', () => {
    alert('Button clicked');
  });

});

</script>

<template>
  
  <Focusguard />
      
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
