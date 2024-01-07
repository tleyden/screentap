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

<template>
  <!-- <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="searchKeyword" placeholder="What are you looking for..." />
    <button type="submit">Search Screenshots</button>
  </form> -->

  <form class="row" @submit.prevent="searchscreenshots">
    <input id="search-screenshots-input" v-model="searchKeyword" placeholder="What are you looking for..." />
    <button type="submit">Search Screenshots</button>
  </form>

  <div>
    <table>
      <tr>
        <th>Image Path</th>
        <th>OCR Text</th>
      </tr>
      <tr v-for="(item, index) in searchScreenshotsResult" :key="index">
        <td>{{ item['image_path'] }}</td>
        <td>{{ item['ocr_text'] }}</td>
      </tr>
    </table>
  </div>

  <!-- <img src="/Users/tleyden/Development/screentap/dataset/2024_01_07_11_10_27.png" alt="Loaded Image" /> -->

</template>
