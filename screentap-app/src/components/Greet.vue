<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const greetMsg = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", { });
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <button type="submit">Save Screenshot</button>
  </form>

  <p>{{ greetMsg }}</p>
</template>
