
<!-- Vue.js script -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

// Keep this as an array because eventually we might request 
// these in blocks
const browseScreenshotsResult = ref([]);

async function browseScreenshots() {
  browseScreenshotsResult.value = await invoke("browse_screenshots", { curId: 0, direction: "backward" });
}

function formatTitle(item: { timestamp: number, ocr_text: string }): string {
  const readableTimestamp = new Date(item.timestamp * 1000).toLocaleString();
  const truncatedText = truncateText(item.ocr_text);
  return `[${readableTimestamp}] OCR Text: ${truncatedText}`;
}

function truncateText(text: string) {
  const maxLength = 500;
  return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
}

function getBase64Image(dynamicBase64: string) {
  return dynamicBase64 ? `data:image/png;base64,${dynamicBase64}` : '';
}

async function getNextPrevScreenshot(direction: string) {

  // Default to the latest screenshot
  let curId = 0;

  // Check if the array is not empty and get the first object's id
  if (browseScreenshotsResult.value.length > 0) {
    curId = parseInt(browseScreenshotsResult.value[0]['id']);
  }

  browseScreenshotsResult.value = await invoke("browse_screenshots", { curId, direction: direction });
  
}

async function onPrevButtonClick() {
  getNextPrevScreenshot("backward");
}

async function onNextButtonClick() {
  getNextPrevScreenshot("forward");
}

browseScreenshots()

</script>

<!-- Vue.js template -->
<template>

  <!-- Flex container for the buttons and header -->
  <div class="flex-container-header">

    <!-- Left Button with "<" (&lt;) -->
    <button class="flex-button-left light-blue-button" @click="onPrevButtonClick">&lt;</button>

    <!-- Header -->
    <h1>Browse screenshots</h1>  

    <!-- Right Button with ">" (&gt;) -->
    <button class="flex-button-right light-blue-button" @click="onNextButtonClick">&gt;</button>

  </div>

  <!-- Flex container for the screenshot -->
  <div class="flex-container">
    <div v-if="browseScreenshotsResult && browseScreenshotsResult.length > 0" class="flex-item">
      <img :src="getBase64Image(browseScreenshotsResult[0]['base64_image'])" alt="Screenshot" :title="formatTitle(browseScreenshotsResult[0])">
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
