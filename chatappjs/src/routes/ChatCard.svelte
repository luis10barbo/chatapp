<script lang="ts">
  import type { Chat } from "./ContainerChatSelector.svelte";

  export let chat: Chat;
  export let click: (e: MouseEvent) => void;
</script>

<button class="chat-card" on:click={click}>
  <header class="chat-card-header">
    <p class="chat-card-name">{chat.chat_name}</p>
  </header>
  <footer class="chat-card-footer">
    <p class="chat-card-time">10:45</p>
    <p class="chat-card-status">
      <span class="chat-card-status-name">Luis</span>
      <span class="chat-card-status-msg">teste teste</span>
    </p>
  </footer>
</button>
