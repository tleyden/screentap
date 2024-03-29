
<!-- Vue.js script -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';

import {
  FwbAccordion,
  FwbAccordionContent,
  FwbAccordionHeader,
  FwbAccordionPanel,
} from 'flowbite-vue';

// Let typescript know about this custom window property
// https://stackoverflow.com/questions/12709074/how-do-you-explicitly-set-a-new-property-on-window-in-typescript
declare global {
    interface Window { __SCREENTAP_SCREENSHOT__: any; }
}

interface ScreenshotEventPayload {
  screenshot_id: number;
  productivity_score: number;
  raw_llm_result_base64: string;
  png_image_path: string;
  job_title: string;
  job_role: string;
}

interface ScreenshotEvent {
  payload: ScreenshotEventPayload;
}

const closeWindow = async () => {
  try {
    await appWindow.close();
  } catch (error) {
    console.error('Error closing window:', error);
  }
};

// Keep a reference to the screenshot id
const screenshotId = ref<number | null>(null);

// screenshotresults array.  Keep as an array because eventually we might request 
// these in blocks
const getScreenshotResult = ref([]);

// Explanation of LLM infer result
const explanationLLMInferResult = ref('');

const pngImagePath = ref('');

const isVisibleExplanationLLMInferResult = ref(false);

const productivityScore = ref(0);

const jobTitle = ref('');

const jobRole = ref('');

async function recordDistractionAlertFeedback(liked: boolean) {
  await invoke("distraction_alert_rating", { liked: liked, screenshotId: screenshotId.value, pngImagePath: pngImagePath.value, jobTitle: jobTitle.value, jobRole: jobRole.value});
  console.log('screenshot id', screenshotId.value);
  closeWindow();
}


async function getScreenshot() {
    // The __SCREENTAP_SCREENSHOT__ window property is set by the rust backend before showing the window
    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('id')) {
        getScreenshotById(parseInt(window.__SCREENTAP_SCREENSHOT__.id));
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.id is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('productivity_score')) {
        productivityScore.value = window.__SCREENTAP_SCREENSHOT__.productivity_score;
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.productivity_score is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('raw_llm_result_base64')) {
        explanationLLMInferResult.value = atob(window.__SCREENTAP_SCREENSHOT__.raw_llm_result_base64);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.raw_llm_result_base64 is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('png_image_path_base_64')) {
        pngImagePath.value = atob(window.__SCREENTAP_SCREENSHOT__.png_image_path_base_64);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.png_image_path_base_64 is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('job_title_base_64')) {
        jobTitle.value = atob(window.__SCREENTAP_SCREENSHOT__.job_title_base_64);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.job_title_base_64 is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('job_role_base_64')) {
        jobRole.value = atob(window.__SCREENTAP_SCREENSHOT__.job_role_base_64);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.job_role_base_64 is not defined');
    }


}

async function getScreenshotById(id: number) {
    screenshotId.value = id;
    getScreenshotResult.value = await invoke("browse_screenshots", { 
            curId: id, 
            direction: "exact" 
        });
}

async function explainLLMInfer() {
    isVisibleExplanationLLMInferResult.value = true;
}


function getBase64Image(dynamicBase64: string) {
  return dynamicBase64 ? `data:image/png;base64,${dynamicBase64}` : '';
}


// Listen for the custom event emitted from Rust
listen('update-screenshot-event', (event: ScreenshotEvent) => {
  productivityScore.value = event.payload.productivity_score;
  explanationLLMInferResult.value = atob(event.payload.raw_llm_result_base64);
  getScreenshotById(event.payload.screenshot_id);
  pngImagePath.value = event.payload.png_image_path;
  jobTitle.value = event.payload.job_title;
  jobRole.value = event.payload.job_role;
});

getScreenshot()

</script>

<!-- Vue.js template -->
<template>

    <div class="flex flex-col items-center justify-center min-h-screen">
        
        <h1 className="text-4xl font-bold mb-2">
        Getting distracted?
        </h1>

        <p className="mt-2 mb-6">
        productivity score: {{ productivityScore }}
        </p>

        <div class="flex space-x-2 mb-8">
            <button class="btn btn-primary" @click="recordDistractionAlertFeedback(true)">👍 Yes</button>
            <button class="btn btn-secondary" @click="recordDistractionAlertFeedback(false)">👎 No</button>
        </div>

        <fwb-accordion class="mt-4 mx-4 mb-4" :open-first-item="false">
            <fwb-accordion-panel>
            <fwb-accordion-header>Details</fwb-accordion-header>
            <fwb-accordion-content>
                
                <div v-if="getScreenshotResult && getScreenshotResult.length > 0" class="flex-item">
                    <img :src="getBase64Image(getScreenshotResult[0]['base64_image'])" alt="Screenshot">
                </div>

                <div class="flex justify-center mt-4">
                    <!-- button: explain reasoning -->
                    <button class="btn btn-primary" @click="explainLLMInfer">🤔 Explain reasoning</button>
                </div>

                <p v-show="isVisibleExplanationLLMInferResult">{{ explanationLLMInferResult }}</p>

            </fwb-accordion-content>
            </fwb-accordion-panel>
        </fwb-accordion>

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
