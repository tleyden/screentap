
<!-- Vue.js script -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

// Keep this as an array because eventually we might request 
// these in blocks
const browseScreenshotsResult = ref([]);

async function browseScreenshots() {
  browseScreenshotsResult.value = await invoke("browse_screenshots", { curId: 3050, direction: "backward" });
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

browseScreenshots()

</script>

<!-- Vue.js template -->
<template>

  <div class="flex-container">
    <div v-for="(item, index) in browseScreenshotsResult" :key="index" class="flex-item">
      <img :src="getBase64Image(item['base64_image'])" alt="Screenshot" :title="formatTitle(item)">
    </div>
  </div>

</template>

<!-- CSS styles -->
<style>

  .flex-container {
    display: flex;
    flex-direction: row; /* or column, depending on how you want to display items */
    flex-wrap: wrap; /* allows items to wrap to the next line */
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

</style>
