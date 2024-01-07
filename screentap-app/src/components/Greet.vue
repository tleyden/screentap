<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const greetMsg = ref("");
const searchKeyword = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("search", { term: searchKeyword.value });
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="searchKeyword" placeholder="What are you looking for..." />
    <button type="submit">Search Screenshots</button>
  </form>

  <p>{{ greetMsg }}</p>

  <img src="/Users/tleyden/Development/screentap/dataset/2024_01_07_11_10_27.png" alt="Loaded Image" />

</template>
