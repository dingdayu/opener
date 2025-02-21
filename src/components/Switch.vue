<!-- filepath: /c:/Users/xiaoyuding/code/opener/src/components/Switch.vue -->
<template>
  <div :class="{'bg-blue-600': checked, 'bg-gray-200': !checked}" class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 ease-in-out cursor-pointer" @click="toggle">
    <span :class="{'translate-x-6': checked, 'translate-x-1': !checked}" class="inline-block h-4 w-4 transform bg-white rounded-full transition-transform duration-200 ease-in-out"></span>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['update:modelValue']);

const checked = ref(props.modelValue);

// 监听 props.modelValue 的变化
watch(() => props.modelValue, (newValue) => {
  checked.value = newValue;
});

function toggle() {
  checked.value = !checked.value;
  emit('update:modelValue', checked.value);
}
</script>