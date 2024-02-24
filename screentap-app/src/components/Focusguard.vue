
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

const isVisibleExplanationLLMInferResult = ref(false);


async function getScreenshot() {
    // The __SCREENTAP_SCREENSHOT__ window property is set by the rust backend before showing the window
    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('id')) {
        getScreenshotById(parseInt(window.__SCREENTAP_SCREENSHOT__.id));
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.id is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('productivity_score')) {
        console.log('Productivity score:', window.__SCREENTAP_SCREENSHOT__.productivity_score);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.productivity_score is not defined');
    }

    if (window.__SCREENTAP_SCREENSHOT__ && window.__SCREENTAP_SCREENSHOT__.hasOwnProperty('raw_llm_result_base64')) {
        console.log('Raw LLM result:', window.__SCREENTAP_SCREENSHOT__.raw_llm_result);
        // decode window.__SCREENTAP_SCREENSHOT__.raw_llm_result from base64 into a string
        explanationLLMInferResult.value = atob(window.__SCREENTAP_SCREENSHOT__.raw_llm_result_base64);
    } else {
        console.error('window.__SCREENTAP_SCREENSHOT__.raw_llm_result_base64 is not defined');
    }

}

async function getScreenshotById(id: number) {
    console.log('getScreenshotById:', id);
    screenshotId.value = id;
    getScreenshotResult.value = await invoke("browse_screenshots", { 
            curId: id, 
            direction: "exact" 
        });
    console.log('/getScreenshotById:', id);
}

// async function explainLLMInfer() {
//     console.log('explainLLMInfer, screenshotId', screenshotId.value);
//     explanationLLMInferResult.value = await invoke("explain_llm_infer", { 
//             screenshotId: screenshotId.value
//         });
//     console.log('/explainLLMInfer.  result', explanationLLMInferResult.value);
    
// }

async function explainLLMInfer() {
    isVisibleExplanationLLMInferResult.value = true;
}


function getBase64Image(dynamicBase64: string) {
  return dynamicBase64 ? `data:image/png;base64,${dynamicBase64}` : '';
}


// Listen for the custom event emitted from Rust
listen('update-screenshot-event', (event) => {
  console.log('Event received from Rust:', event.payload);
  console.log('screenshot_id:', event.payload.screenshot_id);
  console.log('productivity_score:', event.payload.productivity_score);
  console.log('raw_llm_result_base64:', event.payload.raw_llm_result_base64);
  explanationLLMInferResult.value = atob(event.payload.raw_llm_result_base64);
  getScreenshotById(event.payload.screenshot_id);


});


getScreenshot()

</script>

<!-- Vue.js template -->
<template>

    <div class="flex flex-col items-center justify-center min-h-screen">
        
        <h1 className="text-4xl font-bold mb-4">
        Getting distracted??
        </h1>

        <div class="flex space-x-2 mb-4">
            <button class="btn btn-primary" @click="closeWindow">👍 Yes</button>
            <button class="btn btn-secondary" @click="closeWindow">👎 No</button>
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
