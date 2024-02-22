
<!-- Vue.js script -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

// Keep this as an array because eventually we might request 
// these in blocks
const getScreenshotResult = ref([]);

async function getScreenshot() {
    getScreenshotResult.value = await invoke("browse_screenshots", { curId: 5490, direction: "backward" });
}


function getBase64Image(dynamicBase64: string) {
  return dynamicBase64 ? `data:image/png;base64,${dynamicBase64}` : '';
}

getScreenshot()

</script>

<!-- Vue.js template -->
<template>

  <!-- Flex container for the buttons and header -->
  <div class="flex-container-header">


    <!-- Header -->
    <h1>Getting distracted?</h1>  


  </div>

  <!-- Flex container for the screenshot -->
  <div class="flex-container">
    <div v-if="getScreenshotResult && getScreenshotResult.length > 0" class="flex-item">
      <img :src="getBase64Image(getScreenshotResult[0]['base64_image'])" alt="Screenshot">
    </div>
  </div>

</template>

<!-- CSS styles -->
<style>

  .flex-container {
    display: flex;
    flex-direction: row; /* or column, depending on how you want to display items */
    flex-wrap: nowrap; /* allows items to wrap to the next line */
    justify-content: space-around; /* or any other justification you prefer */
  }

  .flex-container-header {
    display: flex;
    flex-direction: row; /* or column, depending on how you want to display items */
    flex-wrap: nowrap; /* allows items to wrap to the next line */
    justify-content: space-around; /* or any other justification you prefer */
  }

  .flex-item {
    margin: 10px; /* adjust as needed for spacing */
    /* additional styles for the flex items */
  }

  .flex-item img {
    width: 100%; /* or any specific size */
    height: auto; /* maintains the aspect ratio */
    /* additional styles for the images */
  }

  .light-blue-button {
    background-color: #c8ecfc; /* Very light blue */
    /* Add other styling as needed */
}

</style>
