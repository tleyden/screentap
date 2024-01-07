
<!-- Vue.js script -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const searchKeyword = ref("");
const searchScreenshotsResult = ref([]);

async function searchscreenshots() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  searchScreenshotsResult.value = await invoke("search_screenshots", { term: searchKeyword.value });
}

</script>

<!-- Vue.js template -->
<template>

  <form class="row" @submit.prevent="searchscreenshots">
    <input id="search-screenshots-input" v-model="searchKeyword" placeholder="What are you looking for..." />
    <button type="submit">Search Screenshots</button>
  </form>

  <div class="flex-container">
    <div v-for="(item, index) in searchScreenshotsResult" :key="index" class="flex-item">
      <img :src="item['file_path']" alt="Screenshot" :title="item['ocr_text']">
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
