<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api'

const username = ref("")
const version = ref("")
const load = ref(false)

const get_versions = () => {
  invoke("get_manifest")
    .then((response) => {
      console.log(response)
    })
}

const download = () => {

}

const play = () => {
  
}
</script>

<template>
  <div class="container">
    <h1>Nomi</h1>
    <input name="username" type="text" class="text_input" v-model.trim="username">
    {{ username }}
    <input name="version" type="text" class="text_input" v-model.trim="version">
    <button @click="get_versions">test</button>
    <button class="button" @click="download">Download</button>
    <button class="button" @click="play">Play</button>
  </div>
</template>